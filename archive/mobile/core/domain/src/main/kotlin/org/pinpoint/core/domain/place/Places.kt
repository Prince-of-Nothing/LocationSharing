package org.pinpoint.core.domain.place

/** Travel mode for routing / navigation (OSRM profiles over OSM data). */
enum class TravelMode { DRIVING, FOOT, BIKE }

/** A searchable place (Nominatim result normalized to a stable model). */
data class SearchPlace(
    val displayName: String,
    val shortName: String,
    val latitude: Double,
    val longitude: Double,
    val category: String = "",
    val type: String = "",
    val osmId: Long? = null,
    val osmType: String? = null,
)

/** POI category for Overpass browse (Organic-Maps style browse). */
data class PoiCategory(
    val id: String,
    val label: String,
    val icon: String,
)

/** A point of interest from Overpass around the map centre. */
data class Poi(
    val osmId: Long,
    val name: String,
    val latitude: Double,
    val longitude: Double,
    val category: String,
    val icon: String,
    val phone: String = "",
    val website: String = "",
    val openingHours: String = "",
    val cuisine: String = "",
)

/** User-saved bookmark (manual star; stored on-device only). */
data class Bookmark(
    val id: String,
    val name: String,
    val latitude: Double,
    val longitude: Double,
    val note: String = "",
    val category: String = "",
    val createdAtMillis: Long = 0L,
)

/** Route step for turn-by-turn lists. */
data class RouteStep(
    val text: String,
    val distanceMeters: Double? = null,
    val durationSeconds: Double? = null,
)

object PoiCategories {
    val ALL: List<PoiCategory> = listOf(
        PoiCategory("food", "Food", "🍽️"),
        PoiCategory("fuel", "Fuel", "⛽"),
        PoiCategory("pharmacy", "Pharmacy", "💊"),
        PoiCategory("hospital", "Health", "🏥"),
        PoiCategory("hotel", "Stay", "🏨"),
        PoiCategory("tourism", "Sights", "🏛️"),
        PoiCategory("water", "Water", "💧"),
        PoiCategory("parking", "Parking", "🅿️"),
        PoiCategory("park", "Parks", "🌳"),
    )
}

/** Geocoding abstraction (Nominatim-backed in :core:data/network). */
interface GeocodingRepository {
    suspend fun search(query: String): List<SearchPlace>
    suspend fun reverse(latitude: Double, longitude: Double): SearchPlace?
}

/** POI abstraction (Overpass-backed). */
interface PoiRepository {
    suspend fun nearby(
        latitude: Double,
        longitude: Double,
        categoryId: String,
        radiusMeters: Int = 1500,
    ): List<Poi>
}

/** On-device bookmarks (DataStore). Never syncs; never stores friend locations. */
interface BookmarkRepository {
    suspend fun list(): List<Bookmark>
    suspend fun add(name: String, latitude: Double, longitude: Double, note: String = "", category: String = ""): Bookmark
    suspend fun remove(id: String)
    suspend fun clear()
}
