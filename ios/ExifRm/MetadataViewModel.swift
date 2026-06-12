import Foundation
import Photos
import PhotosUI
import SwiftUI
import ExifRmRust

@MainActor @Observable
final class MetadataViewModel {
    var imageData: Data?
    var cleanData: Data?
    var isLoading = false
    var errorMessage: String?
    var isStripped = false
    var selectedPhotoItem: PhotosPickerItem?

    func selectImage(_ item: PhotosPickerItem) {
        isLoading = true
        isStripped = false
        cleanData = nil
        selectedPhotoItem = item

        Task {
            let data: Data?

            // Try loading original bytes via PHAssetResourceManager
            // to get exact bytes without iOS re-encoding or adding metadata
            if let identifier = item.itemIdentifier,
               let asset = PHAsset.fetchAssets(withLocalIdentifiers: [identifier], options: nil).firstObject {
                data = await loadOriginalData(from: asset)
            } else {
                // Fallback to Data transferable
                data = try? await item.loadTransferable(type: Data.self)
            }

            guard let data else {
                errorMessage = "Failed to load image"
                isLoading = false
                return
            }

            imageData = data
            isLoading = false
        }
    }

    /// Load the original bytes from a PHAsset via PHAssetResourceManager
    /// to avoid iOS re-encoding or adding metadata.
    private func loadOriginalData(from asset: PHAsset) async -> Data? {
        let resources = PHAssetResource.assetResources(for: asset)
        guard let resource = resources.first(where: { $0.type == .photo }) ?? resources.first else {
            return nil
        }

        return await withCheckedContinuation { continuation in
            let options = PHAssetResourceRequestOptions()
            options.isNetworkAccessAllowed = true

            var accumulated = Data()
            PHAssetResourceManager.default().requestData(for: resource, options: options) { chunk in
                accumulated.append(chunk)
            } completionHandler: { _ in
                continuation.resume(returning: accumulated.isEmpty ? nil : accumulated)
            }
        }
    }

    func stripMetadata() {
        guard let data = imageData else { return }
        isLoading = true

        Task {
            do {
                let clean = try await Task.detached(priority: .userInitiated) {
                    try stripMetadataOwned(input: data)
                }.value
                cleanData = clean
                isStripped = true
                isLoading = false
            } catch {
                errorMessage = error.localizedDescription
                isLoading = false
            }
        }
    }

    func clearError() {
        errorMessage = nil
    }

    func saveToPhotosLibrary() {
        guard let data = cleanData else { return }

        Task {
            do {
                try await PHPhotoLibrary.shared().performChanges {
                    let request = PHAssetCreationRequest.forAsset()
                    let options = PHAssetResourceCreationOptions()
                    options.originalFilename = "cleaned_image.jpg"
                    request.addResource(with: .photo, data: data, options: options)
                }
            } catch {
                errorMessage = "Failed to save to photo library: \(error.localizedDescription)"
            }
        }
    }

    func getTemporaryFileURL() -> URL? {
        guard let data = cleanData else { return nil }
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("cleaned_image_\(Int(Date().timeIntervalSince1970)).jpg")
        try? data.write(to: url)
        return url
    }

    var displayData: Data? {
        isStripped ? cleanData : imageData
    }
}