import SwiftUI
import PhotosUI

struct ContentView: View {
    @State private var viewModel = MetadataViewModel()
    @State private var selectedPhotoItem: PhotosPickerItem?

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading {
                    VStack(spacing: 12) {
                        ProgressView()
                        Text("Processing...")
                            .foregroundStyle(.secondary)
                    }
                } else if viewModel.imageData == nil {
                    VStack(spacing: 16) {
                        Image(systemName: "photo.on.rectangle.angled")
                            .font(.system(size: 48))
                            .foregroundStyle(.secondary)
                        Text("No image selected")
                            .font(.title3)
                            .foregroundStyle(.secondary)
                    }
                } else {
                    ScrollView {
                        VStack(spacing: 16) {
                            imagePreview
                            actionButtons
                        }
                        .padding()
                    }
                }
            }
            .navigationTitle("ExifRm")
            .toolbar {
                if viewModel.imageData == nil {
                    ToolbarItem(placement: .bottomBar) {
                        PhotosPicker(selection: $selectedPhotoItem, matching: .images) {
                            Label("Select Image", systemImage: "plus")
                        }
                    }
                }
            }
            .alert("Error", isPresented: .constant(viewModel.errorMessage != nil)) {
                Button("OK") { viewModel.clearError() }
            } message: {
                Text(viewModel.errorMessage ?? "")
            }
            .onChange(of: selectedPhotoItem) { _, newItem in
                if let newItem {
                    viewModel.selectImage(newItem)
                }
            }
        }
    }

    @ViewBuilder
    private var imagePreview: some View {
        if let data = viewModel.displayData,
           let uiImage = UIImage(data: data) {
            Image(uiImage: uiImage)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(maxHeight: 300)
                .clipShape(RoundedRectangle(cornerRadius: 12))
        }
    }

    @ViewBuilder
    private var actionButtons: some View {
        if !viewModel.isStripped {
            Button(action: viewModel.stripMetadata) {
                Label("Strip Metadata", systemImage: "checkmark")
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        } else {
            VStack(spacing: 12) {
                HStack(spacing: 12) {
                    PhotosPicker(selection: $selectedPhotoItem, matching: .images) {
                        Label("New Image", systemImage: "plus")
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.large)

                    if let url = viewModel.getTemporaryFileURL() {
                        ShareLink(item: url) {
                            Label("Share", systemImage: "square.and.arrow.up")
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.large)
                    }
                }

                Button(action: viewModel.saveToPhotosLibrary) {
                    Label("Save to Gallery", systemImage: "checkmark")
                }
                .buttonStyle(.bordered)
                .controlSize(.large)
            }
        }
    }
}