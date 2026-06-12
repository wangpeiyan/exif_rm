plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
}

android {
    namespace = "com.example.exifrm.library"
    compileSdk = 36

    defaultConfig {
        minSdk = 24

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    // Wire jniLibs from Rust build output
    sourceSets {
        getByName("main") {
            jniLibs.srcDir("../../target/mobile/android/jniLibs")
        }
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

dependencies {
    api("net.java.dev.jna:jna:5.14.0@aar")

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.6.1")
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "com.example.exifrm"
            artifactId = "exif-rm"
            version = "0.1.0"

            afterEvaluate {
                from(components["release"])
            }
        }
    }
}

// Task to build Rust library
tasks.register("buildRust") {
    group = "build"
    description = "Build Rust library for Android"

    doLast {
        val projectRoot = file("../..").absolutePath
        val scriptPath = "$projectRoot/scripts/build-android.sh"

        project.exec {
            workingDir = file(projectRoot)
            environment("ANDROID_NDK_HOME", System.getenv("ANDROID_NDK_HOME")
                ?: "${System.getProperty("user.home")}/Library/Android/sdk/ndk/30.0.14904198")
            commandLine(scriptPath)
        }

        // Copy Kotlin bindings
        val sourceDir = file("../../target/mobile/android/kotlin/uniffi")
        val targetDir = file("src/main/java/uniffi")
        targetDir.mkdirs()

        copy {
            from(sourceDir)
            into(targetDir)
        }
    }
}

// Build Rust before assembling AAR
tasks.named("preBuild") {
    dependsOn("buildRust")
}
