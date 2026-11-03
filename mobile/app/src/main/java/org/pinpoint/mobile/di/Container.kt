package org.pinpoint.mobile.di

import android.content.Context
import org.pinpoint.core.data.AuthRepositoryImpl
import org.pinpoint.core.data.FriendshipRepositoryImpl
import org.pinpoint.core.data.LocationRepositoryImpl
import org.pinpoint.core.data.SessionStore
import org.pinpoint.core.domain.repository.AuthRepository
import org.pinpoint.core.domain.repository.FriendshipRepository
import org.pinpoint.core.domain.repository.LocationRepository
import org.pinpoint.core.map.MapConfig
import org.pinpoint.core.map.OfflineMapManager
import org.pinpoint.core.network.ApiClient

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
 */
class Container(context: Context) {

    private val apiClient = ApiClient(baseUrl = org.pinpoint.mobile.BuildConfig.API_BASE_URL)

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

    /** For app-start session restoration and current-user identification. */
    val session: SessionStore get() = sessionStore
}
