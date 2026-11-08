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
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.pinpoint.core.domain.place.Bookmark
import org.pinpoint.core.domain.place.BookmarkRepository
import org.pinpoint.core.domain.place.Poi
import org.pinpoint.core.domain.place.PoiCategories
import org.pinpoint.core.domain.place.PoiRepository

/** POI + bookmarks screens (Overpass + on-device DataStore). */
class PlacesViewModel(
    private val pois: PoiRepository,
    private val bookmarks: BookmarkRepository,
) : ViewModel() {
    data class UiState(
        val pois: List<Poi> = emptyList(),
        val bookmarks: List<Bookmark> = emptyList(),
        val loading: Boolean = false,
        val error: String? = null,
    )

    private val _state = MutableStateFlow(UiState())
    val state: StateFlow<UiState> = _state.asStateFlow()

    init { refreshBookmarks() }

    fun loadNearby(latitude: Double, longitude: Double, categoryId: String) {
        viewModelScope.launch {
            _state.value = _state.value.copy(loading = true, error = null)
            runCatching { pois.nearby(latitude, longitude, categoryId) }
                .onSuccess { _state.value = _state.value.copy(pois = it, loading = false) }
                .onFailure { e -> _state.value = _state.value.copy(loading = false, error = e.message) }
        }
    }

    fun refreshBookmarks() {
        viewModelScope.launch {
            _state.value = _state.value.copy(bookmarks = runCatching { bookmarks.list() }.getOrDefault(emptyList()))
        }
    }

    fun save(name: String, latitude: Double, longitude: Double, category: String) {
        viewModelScope.launch { bookmarks.add(name, latitude, longitude, category = category); refreshBookmarks() }
    }

    fun deleteBookmark(id: String) {
        viewModelScope.launch { bookmarks.remove(id); refreshBookmarks() }
    }

    companion object {
        fun factory(pois: PoiRepository, bookmarks: BookmarkRepository) = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T =
                PlacesViewModel(pois, bookmarks) as T
        }
    }
}

@Composable
fun PlacesScreen(
    viewModel: PlacesViewModel,
    centerLatitude: Double = 47.0105,
    centerLongitude: Double = 28.8638,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var category = remember { mutableStateOf(PoiCategories.ALL.first().id) }
    Column(modifier = Modifier.fillMaxSize().padding(12.dp)) {
        Text("Places nearby", style = MaterialTheme.typography.titleLarge)
        Text("Overpass · © OpenStreetMap contributors", style = MaterialTheme.typography.bodySmall)
        Row(modifier = Modifier.fillMaxWidth()) {
            Button(
                onClick = { viewModel.loadNearby(centerLatitude, centerLongitude, category.value) },
                modifier = Modifier.weight(1f),
            ) { Text("Load nearby") }
        }
        LazyColumn(modifier = Modifier.weight(1f)) {
            items(PoiCategories.ALL) { c ->
                TextButton(onClick = { category.value = c.id }, modifier = Modifier.fillMaxWidth()) {
                    Text("${c.icon} ${c.label}${if (category.value == c.id) " ●" else ""}")
                }
            }
            if (state.loading) item { LinearProgressIndicator(modifier = Modifier.fillMaxWidth()) }
            state.error?.let { item { Text(it, color = MaterialTheme.colorScheme.error) } }
            items(state.pois) { poi ->
                Card(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
                    Row(modifier = Modifier.padding(12.dp)) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text("${poi.icon} ${poi.name}", style = MaterialTheme.typography.bodyLarge)
                            Text("${poi.latitude}, ${poi.longitude}", style = MaterialTheme.typography.bodySmall)
                        }
                        TextButton(onClick = { viewModel.save(poi.name, poi.latitude, poi.longitude, poi.category) }) {
                            Text("★ Save")
                        }
                    }
                }
            }
            item { Text("Saved places ★ (this device only)", style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(top = 12.dp)) }
            items(state.bookmarks) { bm ->
                Card(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
                    Row(modifier = Modifier.padding(12.dp)) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text("★ ${bm.name}", style = MaterialTheme.typography.bodyLarge)
                            Text("${bm.latitude}, ${bm.longitude}", style = MaterialTheme.typography.bodySmall)
                        }
                        TextButton(onClick = { viewModel.deleteBookmark(bm.id) }) { Text("Delete") }
                    }
                }
            }
        }
    }
}
