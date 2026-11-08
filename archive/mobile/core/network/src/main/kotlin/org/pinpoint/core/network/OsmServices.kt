package org.pinpoint.core.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import org.pinpoint.core.domain.place.Poi
import org.pinpoint.core.domain.place.SearchPlace
import org.pinpoint.core.domain.place.TravelMode
import org.pinpoint.core.domain.route.GeoPoint
import org.pinpoint.core.domain.route.Route
import org.pinpoint.core.domain.route.RoutingEngine
import org.pinpoint.core.domain.place.RouteStep
import retrofit2.http.Field
import retrofit2.http.FormUrlEncoded
import retrofit2.http.GET
import retrofit2.http.Headers
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query
import retrofit2.Retrofit
import java.util.concurrent.TimeUnit
import okhttp3.OkHttpClient

/**
 * OSM public-service clients (user-initiated only).
 *
 *  - Nominatim: geocoding (https://nominatim.openstreetmap.org). Policy:
 *    max 1 req/s per user, valid referer, cached, no bulk. UI debounces.
 *  - OSRM demo: routing (https://router.project-osrm.org). Demo only;
 *    production must self-host OSRM over OSM extracts.
 *  - Overpass: POI browse (https://overpass-api.de). One query per tap,
 *    1.5 km radius, 60 max. Production should self-host/mirror.
 *
 * Friend/shared locations are NEVER sent here. Only user-typed search
 * text or explicitly chosen route endpoints / POI centres.
 */
object OsmEndpoints {
    const val NOMINATIM = "https://nominatim.openstreetmap.org/"
    const val OSRM = "https://router.project-osrm.org/"
    const val OVERPASS = "https://overpass-api.de/api/"
    const val USER_AGENT = "PinpointMobile/1.0 (Android; privacy-first OSM client)"
}

interface NominatimApi {
    @Headers("User-Agent: PinpointMobile/1.0 (Android)", "Accept: application/json")
    @GET("search")
    suspend fun search(
        @Query("q") query: String,
        @Query("format") format: String = "jsonv2",
        @Query("limit") limit: Int = 8,
        @Query("addressdetails") addressdetails: Int = 1,
    ): List<NominatimResult>

    @Headers("User-Agent: PinpointMobile/1.0 (Android)", "Accept: application/json")
    @GET("reverse")
    suspend fun reverse(
        @Query("lat") latitude: Double,
        @Query("lon") longitude: Double,
        @Query("format") format: String = "jsonv2",
        @Query("zoom") zoom: Int = 16,
    ): NominatimResult
}

@Serializable
data class NominatimResult(
    @SerialName("place_id") val placeId: Long? = null,
    @SerialName("display_name") val displayName: String = "",
    @SerialName("lat") val lat: String = "",
    @SerialName("lon") val lon: String = "",
    @SerialName("class") val resultClass: String = "",
    @SerialName("type") val resultType: String = "",
    @SerialName("osm_id") val osmId: Long? = null,
    @SerialName("osm_type") val osmType: String? = null,
)

interface OsrmApi {
    @Headers("Accept: application/json")
    @GET("route/v1/{profile}/{coords}")
    suspend fun route(
        @Path("profile") profile: String,
        @Path("coords") coords: String,
        @Query("overview") overview: String = "full",
        @Query("geometries") geometries: String = "polyline",
        @Query("steps") steps: Boolean = true,
    ): OsrmResponse
}

@Serializable
data class OsrmResponse(
    @SerialName("code") val code: String = "",
    @SerialName("routes") val routes: List<OsrmRoute> = emptyList(),
)

@Serializable
data class OsrmRoute(
    @SerialName("distance") val distance: Double = 0.0,
    @SerialName("duration") val duration: Double = 0.0,
    @SerialName("geometry") val geometry: String = "",
    @SerialName("legs") val legs: List<OsrmLeg> = emptyList(),
)

@Serializable
data class OsrmLeg(
    @SerialName("summary") val summary: String = "",
    @SerialName("steps") val steps: List<OsrmStep> = emptyList(),
)

@Serializable
data class OsrmStep(
    @SerialName("name") val name: String = "",
    @SerialName("distance") val distance: Double = 0.0,
    @SerialName("duration") val duration: Double = 0.0,
    @SerialName("maneuver") val maneuver: OsrmManeuver? = null,
)

@Serializable
data class OsrmManeuver(
    @SerialName("type") val type: String = "",
    @SerialName("modifier") val modifier: String? = null,
    @SerialName("exit") val exit: Int? = null,
)

interface OverpassApi {
    @FormUrlEncoded
    @POST("interpreter")
    suspend fun interpreter(@Field("data") query: String): OverpassResponse
}

@Serializable
data class OverpassResponse(
    @SerialName("elements") val elements: List<OverpassElement> = emptyList(),
)

@Serializable
data class OverpassElement(
    @SerialName("type") val type: String = "",
    @SerialName("id") val id: Long = 0L,
    @SerialName("lat") val lat: Double = 0.0,
    @SerialName("lon") val lon: Double = 0.0,
    @SerialName("tags") val tags: Map<String, String> = emptyMap(),
)

/** Factory for the three OSM services (Retrofit/OkHttp only; no MapLibre). */
class OsmClients {
    private val json = kotlinx.serialization.json.Json { ignoreUnknownKeys = true; coerceInputValues = true }
    private val http = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .addInterceptor { chain ->
            chain.proceed(chain.request().newBuilder().header("User-Agent", OsmEndpoints.USER_AGENT).build())
        }
        .build()

    private fun <T> create(base: String, service: Class<T>): T =
        Retrofit.Builder().baseUrl(base).client(http).addConverterFactory(JsonConverterFactory(json)).build().create(service)

    val nominatim: NominatimApi = create(OsmEndpoints.NOMINATIM, NominatimApi::class.java)
    val osrm: OsrmApi = create(OsmEndpoints.OSRM, OsrmApi::class.java)
    val overpass: OverpassApi = create(OsmEndpoints.OVERPASS, OverpassApi::class.java)
}

/** Maps DTOs to domain models (keeps :core:data thin). */
fun NominatimResult.toDomain(): SearchPlace = SearchPlace(
    displayName = displayName.ifBlank { "Unnamed place" },
    shortName = displayName.substringBefore(",").ifBlank { "Unnamed place" },
    latitude = lat.toDoubleOrNull() ?: 0.0,
    longitude = lon.toDoubleOrNull() ?: 0.0,
    category = resultClass,
    type = resultType,
    osmId = osmId,
    osmType = osmType,
)

fun OsrmResponse.toRoute(from: GeoPoint, to: GeoPoint, mode: TravelMode): Route {
    val r = routes.firstOrNull() ?: throw IllegalStateException("No route found between these points.")
    val points = decodePolyline(r.geometry).ifEmpty { listOf(from, to) }
    val steps = r.legs.flatMap { leg -> leg.steps.map { s -> RouteStep(text = stepText(s), distanceMeters = s.distance, durationSeconds = s.duration) } }
    return Route(
        points = points,
        distanceMeters = r.distance,
        durationSeconds = r.duration,
        instructions = steps.map { it.text },
        steps = steps,
        summary = r.legs.firstOrNull()?.summary ?: "",
        mode = mode,
    )
}

fun stepText(s: OsrmStep): String {
    val t = s.maneuver?.type ?: "continue"
    val verb = when (t) {
        "depart" -> "Depart"
        "arrive" -> "Arrive"
        "turn" -> "Turn"
        "continue", "newname", "notification" -> "Continue"
        "merge" -> "Merge"
        "onramp" -> "Take the ramp"
        "offramp" -> "Exit"
        "fork" -> "Keep"
        "roundabout", "rotary", "roundaboutturn" -> "At the roundabout, take"
        else -> "Continue"
    }
    val name = if (s.name.isNotBlank()) " onto ${s.name}" else ""
    return "$verb$name"
}

fun decodePolyline(encoded: String): List<GeoPoint> {
    if (encoded.isEmpty()) return emptyList()
    val points = mutableListOf<GeoPoint>()
    var index = 0
    var lat = 0
    var lng = 0
    while (index < encoded.length) {
        var shift = 0
        var result = 0
        var byte: Int
        do {
            byte = encoded[index++].code - 63
            result = result or (byte and 0x1f shl shift)
            shift += 5
        } while (byte >= 0x20)
        val dLat = if (result and 1 != 0) (result shr 1).inv() else result shr 1
        lat += dLat
        shift = 0
        result = 0
        do {
            byte = encoded[index++].code - 63
            result = result or (byte and 0x1f shl shift)
            shift += 5
        } while (byte >= 0x20)
        val dLng = if (result and 1 != 0) (result shr 1).inv() else result shr 1
        lng += dLng
        points += GeoPoint(lat / 1e5, lng / 1e5)
    }
    return points
}

/** OSRM-backed routing engine (online; online-only by design). */
class OsrmRoutingEngine(private val api: OsrmApi) : RoutingEngine {
    override val name: String = "osrm (online, OSM data)"
    override val isOfflineCapable: Boolean = false
    override suspend fun route(from: GeoPoint, to: GeoPoint): Route =
        route(from, to, TravelMode.DRIVING)

    suspend fun route(from: GeoPoint, to: GeoPoint, mode: TravelMode): Route {
        val profile = when (mode) {
            TravelMode.DRIVING -> "driving"
            TravelMode.FOOT -> "foot"
            TravelMode.BIKE -> "bike"
        }
        val coords = "${from.longitude},${from.latitude};${to.longitude},${to.latitude}"
        return api.route(profile, coords).toRoute(from, to, mode)
    }
}
