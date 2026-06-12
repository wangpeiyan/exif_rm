package com.example.exifrm

import android.content.ContentValues
import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.exif_rm.Exception as ExifRmException
import uniffi.exif_rm.stripMetadataOwned
import java.io.File
import java.io.FileOutputStream

data class UiState(
    val selectedImageUri: Uri? = null,
    val imageBytes: ByteArray? = null,
    val cleanBytes: ByteArray? = null,
    val isLoading: Boolean = false,
    val error: String? = null,
    val isStripped: Boolean = false
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is UiState) return false
        return selectedImageUri == other.selectedImageUri &&
            isLoading == other.isLoading &&
            error == other.error &&
            isStripped == other.isStripped
    }

    override fun hashCode(): Int {
        var result = selectedImageUri?.hashCode() ?: 0
        result = 31 * result + isLoading.hashCode()
        result = 31 * result + (error?.hashCode() ?: 0)
        result = 31 * result + isStripped.hashCode()
        return result
    }
}

class MetadataViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    fun selectImage(uri: Uri, context: Context) {
        viewModelScope.launch {
            _uiState.value = UiState(isLoading = true, selectedImageUri = uri)

            try {
                val bytes = withContext(Dispatchers.IO) {
                    context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                        ?: throw IllegalArgumentException("Cannot open image")
                }

                _uiState.value = UiState(
                    selectedImageUri = uri,
                    imageBytes = bytes,
                    isLoading = false
                )
            } catch (e: ExifRmException) {
                _uiState.value = UiState(
                    selectedImageUri = uri,
                    error = e.message ?: "Unknown error",
                    isLoading = false
                )
            } catch (e: Exception) {
                _uiState.value = UiState(
                    selectedImageUri = uri,
                    error = e.message ?: "Failed to load image",
                    isLoading = false
                )
            }
        }
    }

    fun stripMetadata() {
        val bytes = _uiState.value.imageBytes ?: return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true)

            try {
                val cleanBytes = withContext(Dispatchers.IO) {
                    stripMetadataOwned(bytes)
                }

                _uiState.value = _uiState.value.copy(
                    cleanBytes = cleanBytes,
                    isLoading = false,
                    isStripped = true
                )
            } catch (e: ExifRmException) {
                _uiState.value = _uiState.value.copy(
                    error = e.message ?: "Failed to strip metadata",
                    isLoading = false
                )
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = e.message ?: "Unknown error",
                    isLoading = false
                )
            }
        }
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }

    fun saveToGallery(context: Context): Boolean {
        val cleanBytes = _uiState.value.cleanBytes ?: return false

        return try {
            val fileName = "cleaned_image_${System.currentTimeMillis()}.jpg"
            val contentValues = ContentValues().apply {
                put(MediaStore.Images.Media.DISPLAY_NAME, fileName)
                put(MediaStore.Images.Media.MIME_TYPE, "image/jpeg")
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_PICTURES + "/ExifRm")
                    put(MediaStore.Images.Media.IS_PENDING, 1)
                }
            }

            val uri = context.contentResolver.insert(
                MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
                contentValues
            ) ?: throw IllegalStateException("Failed to create MediaStore entry")

            context.contentResolver.openOutputStream(uri)?.use { it.write(cleanBytes) }
                ?: throw IllegalStateException("Failed to open output stream")

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                contentValues.clear()
                contentValues.put(MediaStore.Images.Media.IS_PENDING, 0)
                context.contentResolver.update(uri, contentValues, null, null)
            }

            true
        } catch (e: Exception) {
            _uiState.value = _uiState.value.copy(error = "Failed to save to gallery: ${e.message}")
            false
        }
    }

    fun saveToFile(context: Context, fileName: String): Uri? {
        val cleanBytes = _uiState.value.cleanBytes ?: return null

        return try {
            val file = File(context.cacheDir, fileName)
            FileOutputStream(file).use { it.write(cleanBytes) }
            Uri.fromFile(file)
        } catch (e: Exception) {
            _uiState.value = _uiState.value.copy(error = "Failed to save: ${e.message}")
            null
        }
    }

    fun decodeImagePreview(context: Context, bytes: ByteArray?): android.graphics.Bitmap? {
        if (bytes == null) return null
        return BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
    }
}