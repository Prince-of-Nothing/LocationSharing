package org.alwaystogether.feature.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.alwaystogether.core.domain.model.User
import org.alwaystogether.core.domain.repository.ApiException
import org.alwaystogether.core.domain.repository.AuthRepository

/**
 * Login screen state machine. Pure domain-interface dependency: the feature
 * module cannot see Retrofit, OkHttp, DataStore, or the Keystore.
 */
class AuthViewModel(private val authRepository: AuthRepository) : ViewModel() {

    data class UiState(
        val email: String = "",
        val password: String = "",
        val loading: Boolean = false,
        val error: String? = null,
        val loggedInUser: User? = null,
    ) {
        val canSubmit: Boolean
            get() = email.isNotBlank() && password.isNotBlank() && !loading
    }

    private val _state = MutableStateFlow(UiState())
    val state: StateFlow<UiState> = _state.asStateFlow()

    fun onEmailChange(value: String) {
        _state.value = _state.value.copy(email = value, error = null)
    }

    fun onPasswordChange(value: String) {
        _state.value = _state.value.copy(password = value, error = null)
    }

    fun signInOrSignUp() {
        val current = _state.value
        if (!current.canSubmit) return
        _state.value = current.copy(loading = true, error = null)
        viewModelScope.launch {
            try {
                // 1) Try signing in with the credentials.
                val tokens = authRepository.login(current.email, current.password)
                _state.value = _state.value.copy(
                    loading = false,
                    loggedInUser = tokens.user,
                    password = "", // don't keep the password in state
                )
            } catch (loginError: ApiException) {
                if (loginError.httpStatus == 401) {
                    // 2) Credentials rejected → account likely missing: create it.
                    signUp(current.email, current.password)
                } else {
                    _state.value = _state.value.copy(
                        loading = false,
                        error = loginMessage(loginError),
                    )
                }
            } catch (e: Exception) {
                _state.value = _state.value.copy(
                    loading = false,
                    error = "Network error — check your connection",
                )
            }
        }
    }

    /** Step 2 of the combined flow: create the account, then sign in. */
    private fun signUp(email: String, password: String) {
        viewModelScope.launch {
            try {
                val tokens = authRepository.register(email, password)
                _state.value = _state.value.copy(
                    loading = false,
                    loggedInUser = tokens.user,
                    password = "",
                )
            } catch (registerError: ApiException) {
                _state.value = _state.value.copy(
                    loading = false,
                    error = when {
                        // Account exists but the password was wrong.
                        registerError.httpStatus == 409 -> "Wrong password for this account"
                        // Server-side validation (e.g. password too short).
                        registerError.httpStatus == 422 ||
                            registerError.httpStatus == 400 -> registerError.message
                        else -> loginMessage(registerError)
                    },
                )
            } catch (e: Exception) {
                _state.value = _state.value.copy(
                    loading = false,
                    error = "Network error — check your connection",
                )
            }
        }
    }

    private fun loginMessage(e: ApiException): String = when (e.httpStatus) {
        429 -> "Too many attempts, try again later"
        else -> e.message ?: "Unknown error"
    }

    fun login() = signInOrSignUp()

    companion object {
        fun factory(authRepository: AuthRepository) = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T =
                AuthViewModel(authRepository) as T
        }
    }
}
