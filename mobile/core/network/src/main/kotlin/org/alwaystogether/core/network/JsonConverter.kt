package org.alwaystogether.core.network

import java.lang.reflect.Type
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.serializer
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.ResponseBody
import retrofit2.Converter
import retrofit2.Retrofit

/**
 * Retrofit [Converter.Factory] backed by kotlinx.serialization.
 *
 * A small hand-rolled adapter (instead of a third-party one) so the JSON wire
 * format is owned by this module and no extra dependency is required. The
 * [json] instance is configured to ignore unknown keys and coerce nulls to
 * Kotlin defaults (see [ApiClient]).
 */
@OptIn(ExperimentalSerializationApi::class)
class JsonConverterFactory(private val json: Json) : Converter.Factory() {

    override fun responseBodyConverter(
        type: Type,
        annotations: Array<out Annotation>,
        retrofit: Retrofit,
    ): Converter<ResponseBody, *> {
        val serializer = json.serializersModule.serializer(type)
        return object : Converter<ResponseBody, Any> {
            override fun convert(value: ResponseBody): Any =
                json.decodeFromString(serializer, value.string())
        }
    }

    override fun requestBodyConverter(
        type: Type,
        parameterAnnotations: Array<out Annotation>,
        methodAnnotations: Array<out Annotation>,
        retrofit: Retrofit,
    ): Converter<*, RequestBody> {
        val serializer = json.serializersModule.serializer(type)
        return object : Converter<Any, RequestBody> {
            override fun convert(value: Any): RequestBody =
                json.encodeToString(serializer, value)
                    .toRequestBody("application/json".toMediaType())
        }
    }
}
