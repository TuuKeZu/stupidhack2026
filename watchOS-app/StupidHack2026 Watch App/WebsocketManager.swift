import Foundation

actor WebSocketManager {
    private var task: URLSessionWebSocketTask?
    private let session = URLSession(configuration: .default)
    private let url: URL
    private var retryCount = 0
    private let maxRetry = 5
    
    public var isConnected: Bool { task != nil }
    
    var onMessage: ((String) -> Void)?
    
    init(url: URL) {
        self.url = url
    }
    
    func connect() {
        retryCount = 0
        establishConnection()
    }
    
    private func establishConnection() {
        task?.cancel(with: .goingAway, reason: nil)
        let newTask = session.webSocketTask(with: url)
        self.task = newTask
        newTask.resume()
        retryCount = 0
        listen()
    }
    
    func setOnMessage(_ handler: @escaping (String) -> Void) {
          self.onMessage = handler
      }
    
    private func listen() {
        task?.receive { [weak self] result in
            Task {
                guard let self else { return }
                switch result {
                case .success(let message):
                    switch message {
                    case .string(let text):
                        await self.handleMessage(text)
                    case .data(let data):
                        await self.handleMessage(String(data: data, encoding: .utf8) ?? "")
                    @unknown default: break
                    }
                    await self.listen() // keep listening
                case .failure:
                    
                    await self.reconnect()
                }
            }
        }
    }
    
    private func handleMessage(_ text: String) {
        print(text)
        print("\n")
        onMessage?(text)
    }
    
    private func reconnect() {
        guard retryCount < maxRetry else {
            print("Max retries reached")
            return
        }
        let delay = pow(2.0, Double(retryCount)) // 1, 2, 4, 8, 16 sec
        retryCount += 1
        print("Reconnecting in \(delay)s (attempt \(retryCount))")
        
        Task {
            try await Task.sleep(for: .seconds(delay))
            establishConnection()
        }
    }
    
    func send(_ text: String) {
        task?.send(.string(text)) { error in
            if let error { print("Send error: \(error)") }
        }
    }
    
    func disconnect() {
        retryCount = maxRetry // prevent reconnect
        task?.cancel(with: .normalClosure, reason: nil)
    }
}
