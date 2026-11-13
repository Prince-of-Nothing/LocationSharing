package org.pinpoint.mobile.di

import android.content.Context
import org.pinpoint.core.data.AuthRepositoryImpl
import org.pinpoint.core.data.BookmarkRepositoryImpl
import org.pinpoint.core.data.FriendshipRepositoryImpl
import org.pinpoint.core.data.GeocodingRepositoryImpl
import org.pinpoint.core.data.LocationRepositoryImpl
import org.pinpoint.core.data.PoiRepositoryImpl
import org.pinpoint.core.data.SessionStore
import org.pinpoint.core.domain.repository.AuthRepository
import org.pinpoint.core.domain.repository.FriendshipRepository
import org.pinpoint.core.domain.repository.LocationRepository
import org.pinpoint.core.map.DefaultRoutingEngine
import org.pinpoint.core.map.MapConfig
import org.pinpoint.core.map.OfflineMapManager
import org.pinpoint.core.network.ApiClient
import org.pinpoint.core.network.OsmClients
import org.pinpoint.core.network.OsrmRoutingEngine

/**
 * Manual dependency container (composition root). One instance per process,
 * created in [org.pinpoint.mobile.MobileApp].
 *
 * Layering contract (enforced by module boundaries, not convention):
 *  - :feature:* modules depend on :core:domain interfaces only.
 *  - :core:data implements them over :core:network + Keystore storage.
 *  - :core:network is the only module that knows Retrofit/OkHttp.
 *  - :core:map is the only module that knows MapLibre.
 *  - No DI framework: wiring is explicit and visible in this one file.
 *
 * OSM wiring (Organic-Maps / Magic-Earth parity over OSM data):
 *  - Nominatim geocoding, OSRM routing (online) + straight-line fallback,
 *    Overpass POI browse — all user-initiated, never friend locations.
 *  - Bookmarks: DataStore JSON on-device only (no sync/server).
 *  - Map styles: OpenFreeMap vector styles (MapLayers); offline packs
 *    via OfflineMapManager. Attribution is enforced in MapScreen.
 */
class Container(context: Context) {

    private val apiClient = ApiClient(baseUrl = org.pinpoint.mobile.BuildConfig.API_BASE_URL)
    private val osmClients = OsmClients()

    private val sessionStore = SessionStore(context)

    // Network layer reads credentials from the session store's memory cache.
    init {
        apiClient.tokenProvider = { sessionStore.accessToken() }
        apiClient.csrfProvider = { sessionStore.csrfToken() }
    }

    val authRepository: AuthRepository = AuthRepositoryImpl(apiClient.authApi, sessionStore)
    val friendshipRepository: FriendshipRepository = FriendshipRepositoryImpl(apiClient.friendshipApi)
    val locationRepository: LocationRepository = LocationRepositoryImpl(apiClient.locationApi)

    /** Offline map packs (download/list/delete) over MapLibre's OfflineManager. */
    val offlineMaps = OfflineMapManager(context)

    /** Map tile/style configuration (privacy-preserving default). */
    val mapConfig: MapConfig = MapConfig.DEFAULT

    /** OSM search (Nominatim) — user-initiated. */
    val geocodingRepository = GeocodingRepositoryImpl(osmClients.nominatim)

    /** OSM POI browse (Overpass) — user-initiated. */
    val poiRepository = PoiRepositoryImpl(osmClients.overpass)

    /** On-device bookmarks (no sync, no friend data). */
    val bookmarkRepository = BookmarkRepositoryImpl(context)

    /** Online routing (OSRM); straight-line stub stays as offline fallback. */
    val routingEngine: org.pinpoint.core.domain.route.RoutingEngine =
        OsrmRoutingEngine(osmClients.osrm)

    /** Offline-safe fallback engine (straight-line estimate). */
    val offlineRoutingEngine: org.pinpoint.core.domain.route.RoutingEngine =
        DefaultRoutingEngine()

    /** For app-start session restoration and current-user identification. */
    val session: SessionStore get() = sessionStore
}
