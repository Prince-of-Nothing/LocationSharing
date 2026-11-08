package org.pinpoint.core.domain.route

/** A geographic coordinate (WGS84 degrees). */
data class GeoPoint(val latitude: Double, val longitude: Double) {
    init {
        require(latitude in -90.0..90.0) { "latitude out of range: $latitude" }
        require(longitude in -180.0..180.0) { "longitude out of range: $longitude" }
    }
}

/** A computed route between two points. */
data class Route(
    val points: List<GeoPoint>,
    val distanceMeters: Double,
    val durationSeconds: Double,
    val instructions: List<String> = emptyList(),
    val steps: List<org.pinpoint.core.domain.place.RouteStep> = emptyList(),
    val summary: String = "",
    val mode: org.pinpoint.core.domain.place.TravelMode = org.pinpoint.core.domain.place.TravelMode.DRIVING,
)

/**
 * Swappable routing abstraction (board T14: offline navigation is delivered
 * as an intent export + engine stub; a real offline engine — GraphHopper,
 * Valhalla — plugs in here later without touching features or the map layer).
 */
interface RoutingEngine {
    /** Human-readable engine name (for UI/debug). */
    val name: String

    /** True when the engine can route without network (offline-capable). */
    val isOfflineCapable: Boolean

    /**
     * Computes a route. Implementations may be suspend/network-bound
     * (online engines) or immediate (offline engines over local data).
     */
    suspend fun route(from: GeoPoint, to: GeoPoint): Route
}
