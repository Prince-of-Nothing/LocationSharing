package org.pinpoint.core.data

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.first
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import org.pinpoint.core.domain.place.Bookmark
import org.pinpoint.core.domain.place.BookmarkRepository
import org.pinpoint.core.domain.place.GeocodingRepository
import org.pinpoint.core.domain.place.Poi
import org.pinpoint.core.domain.place.PoiRepository
import org.pinpoint.core.domain.place.SearchPlace
import org.pinpoint.core.network.NominatimApi
import org.pinpoint.core.network.OverpassApi
import org.pinpoint.core.network.toDomain

private val Context.bookmarksDataStore by preferencesDataStore(name = "bookmarks")

/**
 * OSM repositories over Nominatim + Overpass. User-initiated only;
 * friend/shared locations never reach these services.
 */
class GeocodingRepositoryImpl(private val api: NominatimApi) : GeocodingRepository {
    override suspend fun search(query: String): List<SearchPlace> =
        api.search(query.trim()).map { it.toDomain() }
            .filter { it.latitude != 0.0 || it.longitude != 0.0 }
            .sortedBy { it.displayName }

    override suspend fun reverse(latitude: Double, longitude: Double): SearchPlace? =
        api.reverse(latitude, longitude).toDomain()
}

class PoiRepositoryImpl(private val api: OverpassApi) : PoiRepository {
    override suspend fun nearby(
        latitude: Double,
        longitude: Double,
        categoryId: String,
        radiusMeters: Int,
    ): List<Poi> {
        val r = radiusMeters.coerceIn(200, 5000)
        val fragment = poiFragment(categoryId) ?: return emptyList()
        val q = "[out:json][timeout:25];(" +
            fragment.replace("{r}", r.toString())
                .replace("{lat}", latitude.toString())
                .replace("{lon}", longitude.toString()) +
            ");out center 60;"
        return api.interpreter(q).elements.map { e ->
            val tags = e.tags
            Poi(
                osmId = e.id,
                name = tags["name"] ?: tags["brand"] ?: tags["operator"] ?: "(unnamed)",
                latitude = e.lat,
                longitude = e.lon,
                category = categoryId,
                icon = poiIcon(categoryId),
                phone = tags["phone"] ?: tags["contact:phone"] ?: "",
                website = tags["website"] ?: tags["contact:website"] ?: "",
                openingHours = tags["opening_hours"] ?: "",
                cuisine = tags["cuisine"] ?: "",
            )
        }
    }

    private fun poiFragment(id: String): String? = when (id) {
        "food" -> """node["amenity"="restaurant"](around:{r},{lat},{lon});node["amenity"="cafe"](around:{r},{lat},{lon});node["amenity"="fast_food"](around:{r},{lat},{lon});"""
        "fuel" -> """node["amenity"="fuel"](around:{r},{lat},{lon});"""
        "pharmacy" -> """node["amenity"="pharmacy"](around:{r},{lat},{lon});"""
        "hospital" -> """node["amenity"="hospital"](around:{r},{lat},{lon});node["amenity"="doctors"](around:{r},{lat},{lon});"""
        "hotel" -> """node["tourism"="hotel"](around:{r},{lat},{lon});node["tourism"="hostel"](around:{r},{lat},{lon});node["tourism"="guest_house"](around:{r},{lat},{lon});"""
        "tourism" -> """node["tourism"="museum"](around:{r},{lat},{lon});node["tourism"="attraction"](around:{r},{lat},{lon});node["tourism"="viewpoint"](around:{r},{lat},{lon});"""
        "water" -> """node["amenity"="drinking_water"](around:{r},{lat},{lon});node["amenity"="toilets"](around:{r},{lat},{lon});"""
        "parking" -> """node["amenity"="parking"](around:{r},{lat},{lon});"""
        "park" -> """node["leisure"="park"](around:{r},{lat},{lon});node["leisure"="playground"](around:{r},{lat},{lon});"""
        else -> null
    }

    private fun poiIcon(id: String): String = when (id) {
        "food" -> "🍽️"
        "fuel" -> "⛽"
        "pharmacy" -> "💊"
        "hospital" -> "🏥"
        "hotel" -> "🏨"
        "tourism" -> "🏛️"
        "water" -> "💧"
        "parking" -> "🅿️"
        "park" -> "🌳"
        else -> "•"
    }
}

@Serializable
private data class StoredBookmark(
    val id: String,
    val name: String,
    val latitude: Double,
    val longitude: Double,
    val note: String,
    val category: String,
    val createdAtMillis: Long,
)

private fun StoredBookmark.toDomain() =
    Bookmark(id, name, latitude, longitude, note, category, createdAtMillis)

private fun Bookmark.toStored() =
    StoredBookmark(id, name, latitude, longitude, note, category, createdAtMillis)

/** On-device bookmarks: DataStore JSON. No sync, no server, no friend data. */
class BookmarkRepositoryImpl(private val context: Context) : BookmarkRepository {
    private val json = Json { ignoreUnknownKeys = true }
    private val ds = context.bookmarksDataStore

    override suspend fun list(): List<Bookmark> {
        val raw = ds.data.first()[KEY] ?: return emptyList()
        return runCatching {
            val arr = json.decodeFromString(kotlinx.serialization.builtins.ListSerializer(StoredBookmark.serializer()), raw)
            arr.map { it.toDomain() }.sortedByDescending { it.createdAtMillis }
        }.getOrDefault(emptyList())
    }

    override suspend fun add(
        name: String,
        latitude: Double,
        longitude: Double,
        note: String,
        category: String,
    ): Bookmark {
        val bm = Bookmark(
            id = "bm_${System.currentTimeMillis()}",
            name = name.ifBlank { "Saved place" },
            latitude = latitude,
            longitude = longitude,
            note = note,
            category = category,
            createdAtMillis = System.currentTimeMillis(),
        )
        val next = (list() + bm).sortedByDescending { it.createdAtMillis }.map { it.toStored() }
        ds.edit { it[KEY] = json.encodeToString(kotlinx.serialization.builtins.ListSerializer(StoredBookmark.serializer()), next) }
        return bm
    }

    override suspend fun remove(id: String) {
        val next = list().filter { it.id != id }.map { it.toStored() }
        ds.edit { it[KEY] = json.encodeToString(kotlinx.serialization.builtins.ListSerializer(StoredBookmark.serializer()), next) }
    }

    override suspend fun clear() {
        ds.edit { it.remove(KEY) }
    }

    private companion object {
        val KEY = stringPreferencesKey("bookmarks_json")
    }
}
