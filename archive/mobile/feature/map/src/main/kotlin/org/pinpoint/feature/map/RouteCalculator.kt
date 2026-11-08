/**
 * Simple direction calculator: straight-line distance + estimated travel times.
 *
 * Not a turn-by-turn router (MapLibre Native has no routing engine); this
 * provides quick distance estimates for planning: walking, bike, car, transit.
 */
package org.pinpoint.feature.map

import org.pinpoint.core.map.MapLatLng

/** Average speed estimates (km/h). */
object RouteSpeeds {
    const val WALK = 5.0
    const val BIKE = 15.0
    const val CAR = 40.0
    const val TRANSIT = 25.0
}

fun distanceKm(from: MapLatLng, to: MapLatLng): Double {
    val R = 6371.0
    val dLat = Math.toRadians(to.latitude - from.latitude)
    val dLng = Math.toRadians(to.longitude - from.longitude)
    val a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
            Math.cos(Math.toRadians(from.latitude)) * Math.cos(Math.toRadians(to.latitude)) *
            Math.sin(dLng / 2) * Math.sin(dLng / 2)
    val c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
    return R * c
}

fun timeMinutes(km: Double, speedKmh: Double): Int =
    if (speedKmh <= 0 || km <= 0) 0 else (km / speedKmh * 60.0).toInt()
