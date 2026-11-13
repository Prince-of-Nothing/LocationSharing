package org.pinpoint.feature.map

import android.content.Intent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.pinpoint.core.domain.place.TravelMode
import org.pinpoint.core.domain.route.GeoPoint
import org.pinpoint.core.domain.route.Route
import org.pinpoint.core.domain.route.RoutingEngine

/** Route planner over OSRM (online) with geo: + OSM hand-off. */
class RouteViewModel(private val engine: RoutingEngine) : ViewModel() {
    data class UiState(
        val route: Route? = null,
        val computing: Boolean = false,
        val error: String? = null,
    )

    private val _state = MutableStateFlow(UiState())
    val state: StateFlow<UiState> = _state.asStateFlow()

    fun compute(fromText: String, toText: String, mode: TravelMode) {
        viewModelScope.launch {
            _state.value = _state.value.copy(computing = true, error = null)
            runCatching {
                val from = parse(fromText) ?: error("From must be \"lat,lon\"")
                val to = parse(toText) ?: error("To must be \"lat,lon\"")
                engine.route(from, to)
            }.onSuccess { _state.value = UiState(route = it) }
                .onFailure { e -> _state.value = _state.value.copy(computing = false, error = e.message) }
        }
    }

    fun clear() { _state.value = UiState() }

    private fun parse(text: String): GeoPoint? {
        val parts = text.trim().split(',', ' ').mapNotNull { it.toDoubleOrNull() }
        if (parts.size < 2) return null
        return runCatching { GeoPoint(parts[0], parts[1]) }.getOrNull()
    }

    companion object {
        fun factory(engine: RoutingEngine) = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T =
                RouteViewModel(engine) as T
        }
    }
}

@Composable
fun RoutePlannerScreen(viewModel: RouteViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val context = LocalContext.current
    var from = remember { mutableStateOf("") }
    var to = remember { mutableStateOf("") }
    var mode = remember { mutableStateOf(TravelMode.DRIVING) }
    Column(modifier = Modifier.fillMaxSize().padding(12.dp)) {
        Text("Plan route", style = MaterialTheme.typography.titleLarge)
        Text("OSRM over OpenStreetMap data · © OpenStreetMap contributors", style = MaterialTheme.typography.bodySmall)
        OutlinedTextField(
            value = from.value, onValueChange = { from.value = it },
            label = { Text("From (lat,lon)") }, modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
        )
        OutlinedTextField(
            value = to.value, onValueChange = { to.value = it },
            label = { Text("To (lat,lon)") }, modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
        )
        Row {
            TravelMode.entries.forEach { m ->
                OutlinedButton(onClick = { mode.value = m }, modifier = Modifier.padding(end = 8.dp)) {
                    Text(if (mode.value == m) "● ${m.name.lowercase()}" else m.name.lowercase())
                }
            }
        }
        Button(
            onClick = { viewModel.compute(from.value, to.value, mode.value) },
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Compute route") }
        if (state.computing) LinearProgressIndicator(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp))
        state.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        state.route?.let { route ->
            Text(
                "${"%.1f".format(route.distanceMeters / 1000)} km · ${"%.0f".format(route.durationSeconds / 60)} min",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(top = 8.dp),
            )
            route.steps.take(12).forEach { step -> Text("• ${step.text}", style = MaterialTheme.typography.bodySmall) }
            OutlinedButton(
                onClick = {
                    val dest = route.points.last()
                    runCatching {
                        context.startActivity(
                            Intent.createChooser(
                                navigationIntent(org.pinpoint.core.map.MapLatLng(dest.latitude, dest.longitude), from = null),
                                "Navigate with",
                            ),
                        )
                    }
                },
                modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
            ) { Text("Open in nav app (geo:)") }
            OutlinedButton(onClick = { viewModel.clear() }, modifier = Modifier.fillMaxWidth()) { Text("Clear") }
        }
    }
}
