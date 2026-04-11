import SwiftUI
import Observation
internal import System

@main
struct StupidHack2026_Watch_AppApp: App {
    //@State private var store = AppStore(url: URL(string: "ws://172.20.10.7:10469/ws")!)
    @State private var store = AppStore(url: URL(string: "ws://46.62.167.18:3030/ws/client")!)

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(store)
        }
    }
}
