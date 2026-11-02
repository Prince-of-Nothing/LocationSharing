package org.alwaystogether.feature.map

/**
 * Safety / police reference dataset (demo / reference layer).
 *
 * This is a static reference for testing the alert feature; in production it
 * would connect to a live police/safety API or government open-data feed.
 * No live tracking, no personal data — only public-safety reference points.
 */
object SafetyAlerts {
    data class Alert(
        val id: String,
        val label: String,
        val lat: Double,
        val lon: Double,
        val radiusMeters: Int,
        val message: String,
    )

    // Reference points for demo / testing (near example coordinates)
    val referenceAlerts = listOf(
        Alert("P01", "Police Station — Central", 46.7712, 23.5859, 500, "Public safety post nearby."),
        Alert("P02", "Emergency Evacuation Point", 46.7700, 23.5900, 300, "Evacuation / shelter direction available."),
        Alert("P03", "Traffic Control / Construction Zone", 46.7695, 23.6020, 200, "Direction course may vary — check live update."),
    )
}
