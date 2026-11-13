package org.pinpoint.feature.map

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.pinpoint.core.domain.place.GeocodingRepository
import org.pinpoint.core.domain.place.SearchPlace

/** Search screen (Nominatim over OSM data) — user-initiated, debounced. */
class SearchViewModel(private val geocoding: GeocodingRepository) : ViewModel() {
    data class UiState(
        val results: List<SearchPlace> = emptyList(),
        val searching: Boolean = false,
        val error: String? = null,
    )

    private val _state = MutableStateFlow(UiState())
    val state: StateFlow<UiState> = _state.asStateFlow()
    private var job: Job? = null

    fun query(text: String) {
        job?.cancel()
        val q = text.trim()
        if (q.length < 3) {
            _state.value = UiState()
            return
        }
        job = viewModelScope.launch {
            _state.value = _state.value.copy(searching = true, error = null)
            delay(350)
            runCatching { geocoding.search(q) }
                .onSuccess { results -> _state.value = UiState(results = results, searching = false) }
                .onFailure { e -> _state.value = UiState(error = e.message ?: "Search failed") }
        }
    }

    companion object {
        fun factory(geocoding: GeocodingRepository) = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T =
                SearchViewModel(geocoding) as T
        }
    }
}

@Composable
fun SearchScreen(
    viewModel: SearchViewModel,
    onPick: (SearchPlace) -> Unit,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val context = LocalContext.current
    var text = androidx.compose.runtime.remember { androidx.compose.runtime.mutableStateOf("") }
    Column(modifier = Modifier.fillMaxSize().padding(12.dp)) {
        Text("Search OpenStreetMap", style = MaterialTheme.typography.titleLarge)
        Text("Powered by Nominatim · © OpenStreetMap contributors", style = MaterialTheme.typography.bodySmall)
        OutlinedTextField(
            value = text.value,
            onValueChange = { text.value = it; viewModel.query(it) },
            label = { Text("Place, café, pharmacy…") },
            modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
            singleLine = true,
        )
        if (state.searching) LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
        state.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        LazyColumn {
            items(state.results) { place ->
                Card(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
                    Row(modifier = Modifier.padding(12.dp)) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(place.shortName, style = MaterialTheme.typography.bodyLarge)
                            Text(place.displayName, style = MaterialTheme.typography.bodySmall)
                        }
                        Button(onClick = {
                            // Hand-off is a geo: intent from the caller; keep this composable testable.
                            runCatching {
                                val intent = navigationIntent(org.pinpoint.core.map.MapLatLng(place.latitude, place.longitude), from = null)
                                context.startActivity(android.content.Intent.createChooser(intent, "Open in maps"))
                            }
                            onPick(place)
                        }) { Text("Open") }
                    }
                }
            }
        }
        if (state.results.isEmpty() && !state.searching) {
            Text("Type at least 3 characters. Results come from live OSM search.", style = MaterialTheme.typography.bodySmall)
        }
    }
}
