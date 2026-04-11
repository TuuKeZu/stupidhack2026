import Observation
import Foundation


@Observable
final class AppStore {
    var isConnected = false
    var lastMessage = ""
    var bacTarget = 0.00
    var bacCurrent = 0.00
    var requireUpdate = true
    var history: [Double] = []
    var future: [Double] = []
    let socket: WebSocketManager

    init(url: URL) {
        self.socket = WebSocketManager(url: url)
    }

    func connect() {
        Task {
            await socket.setOnMessage { [weak self] message in
                guard let self else { return }

                let data = Data(message.utf8)
                if let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
                    Task { @MainActor in
                        let type = obj["type"] as? String
                        if type == "status" {
                            self.bacCurrent =  (obj["current"] as! Double) * 1000.0
                            self.bacTarget = (obj["target"] as! Double) * 1000.0
                            self.requireUpdate = obj["update"] as! Bool
                            var estimate = obj["estimate"] as! [String: Any]
                            var history = estimate["history"] as! [Double?]
                            var future = estimate["future"] as! [Double?]
                            self.history = history.map { $0 == nil ? 0.0 : $0! * 1000.0 }
                            self.future = future.map { $0 == nil ? 0.0 : $0! * 1000.0 }
                        }
                    }
                }
            }

            await socket.connect()

            await MainActor.run {
                self.isConnected = true
            }
        }
    }
}
