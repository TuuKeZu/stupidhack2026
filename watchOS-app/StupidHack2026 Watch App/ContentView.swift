import SwiftUI
import Charts

struct ContentView: View {
    @Environment(AppStore.self) private var store

        var body: some View {
            if store.isConnected {
                NavigationStack {
                    MainView()
                }
            } else {
                ConnectView()
            }
        }
}


struct ConnectView: View {
    @Environment(AppStore.self) private var store

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "antenna.radiowaves.left.and.right")
                .imageScale(.large)
                .foregroundStyle(.tint)

            Button("Connect") {
                store.connect()
            }
            .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}

struct MainView: View {
    var body: some View {
        TabView {
            View1()
            GraphView()
            View3()
        }
        .tabViewStyle(.page)
        .navigationBarBackButtonHidden(true)
    }
}

struct GraphView: View {
    @Environment(AppStore.self) private var store

    struct Point: Identifiable {
        let id = UUID()
        let x: Double
        let y: Double
    }

    private let data: [Point] = (0..<40).map { i in
        let x = Double(i) / 5
        let y = (sin(x) + 1.0)/2
        return .init(x: x, y: y)
    }

    var body: some View {
        Chart {
            // Past and present (<= Now): solid line (series: "past")
            ForEach(
                Array(store.history
                    .enumerated()),
                id: \.offset
            ) { index, value in
                LineMark(
                    x: .value("Index", index),
                    y: .value("Value", value),
                    series: .value("Segment", "past")
                )
                .interpolationMethod(.linear)
                .lineStyle(StrokeStyle(lineWidth: 2))
            }
            
            ForEach(
                Array(store.future
                    .enumerated()),
                id: \.offset
            ) { index, value in
                LineMark(
                    x: .value("Index", index + 20),
                    y: .value("Value", value),
                    series: .value("Segment", "future")
                )
                .interpolationMethod(.linear)
                .lineStyle(StrokeStyle(lineWidth: 2, dash: [4, 4]))
            }

            // Future (> Now): dashed line (series: "future")
            /*
            ForEach(store.future.map { $0 }) { point in
                LineMark(
                    x: .value("Index", point.x),
                    y: .value("Value", point.y),
                    series: .value("Segment", "future")
                )
                .interpolationMethod(.linear)
                .lineStyle(StrokeStyle(lineWidth: 2, dash: [4, 4]))
            }*/
        }
        .chartXAxis {
           
            AxisMarks(values: [0, 20, 39]) { value in
                AxisGridLine()
                AxisValueLabel {
                    if let v = value.as(Int.self) {
                        if v == 0 {
                            Text("-2h")
                        } else if v == 20 {
                            Text("Now")
                        } else if v == 39 {
                            Text("+2h")
                        }
                    }
                }
            }
        }
        .chartYAxis {
            // Determine current max Y from data, then set upper bound to ceil(current + 1.0)
            let currentMaxY = (store.history + store.future).max() ?? 0.0
            let upper = max(1, Int(ceil(currentMaxY + 1.0)))
            let ticks: [Double] = Array(0...upper).map { Double($0) }

            AxisMarks(position: .trailing, values: ticks) { value in
                AxisGridLine()
                AxisValueLabel()
            }
        }
    }
}

struct View3: View {
    @Environment(AppStore.self) private var store

    @Environment(\.dismiss) private var dismiss


    var body: some View {
        VStack(spacing: 12) {
            Button("Reset", role: .destructive) {
                do {
                    let dict: [String: Any] = [
                        "type": "reset",
                    ]
                    let data = String(data: try JSONSerialization.data(withJSONObject: dict), encoding: .utf8)!
                    Task {
                        await store.socket.send(data)
                    }
                } catch {}
                dismiss()
            }
            Button("Disconnect", role: .destructive) {
                Task {
                    store.socket.disconnect()
                }
                dismiss()
            }
        }
        .padding()
    }
}

struct View1: View {
    @Environment(AppStore.self) private var store
    
    var ringColor: Color = Color(UIColor(hex: "#DE9E13"))
    var step: Double = 0.01
    var minValue: Double = 0.0
    var maxValue: Double = 5.0
    @State private var value: Double = 0.10
    
    @State private var dismissPopup: Bool = false
    
    private var showPopup: Bool { store.requireUpdate && !dismissPopup }

    private var snappedValue: Double {
        (value / step).rounded() * step
    }
    
    private var snappedCurrentValue: Double {
        (store.bacCurrent / step).rounded() * step
    }
    
    @State private var blinking = false
    
    private var progress: Double {
        let target = store.bacTarget
        print(store.bacCurrent)
        if target < 0.01 { return 1.0 }
        let ratio = store.bacCurrent / target
        return min(max(ratio, 0.00), 1.00)
    }
    
    private func updateValue() -> Void {
        print("Send update")
        store.bacTarget = value
        do {
            let dict: [String: Any] = [
                "type": "target",
                "value": value / 1000.0,
            ]
            let data = String(data: try JSONSerialization.data(withJSONObject: dict), encoding: .utf8)!
            Task {
        
                await store.socket.send(data)
            }
        } catch {}
    }

    var body: some View {
        @Bindable var store = store
        
        ZStack {
            Circle()
                .stroke(ringColor.opacity(0.2), lineWidth: 8)

            Circle()
                .trim(from: 0, to: progress)
                .stroke(
                    ringColor,
                    style: StrokeStyle(lineWidth: 8, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))

            VStack(spacing: 2) {
                Button {
                    value = min(maxValue, snappedValue + step)
                    updateValue()
                } label: {
                    Image(systemName: "plus")
                        .font(.largeTitle)
                        .frame(width: 40, height: 40)
                }
                .buttonStyle(.plain)

                Text(String(format: "%.2f", snappedValue))
                    .font(.title2)
                    .monospacedDigit()
                
                Text(String(format: "Current: %.2f", snappedCurrentValue))
                    .font(.custom("", fixedSize: 10))
                    .monospacedDigit()

                Button {
                    value = max(minValue, snappedValue - step)
                    updateValue()
                } label: {
                    Image(systemName: "minus")
                        .font(.largeTitle)
                        .frame(width: 40, height: 40)
                }
                .buttonStyle(.plain)
            }
            
            VStack {
                VStack {
                    VStack {
                        HStack {
                            Image(systemName: "wineglass")
                                .imageScale(.large)
                                .foregroundStyle(.white)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(2)
                        Text("Time to calibrate!")
                            .padding([.bottom], 3)
                        Text("Use the breathalyzer to calibrate the app.")
                            .font(.custom("", size: 10))
                            .multilineTextAlignment(.center)
                            .padding([.bottom], 6)
                    }
                    .opacity(blinking ? 0.5 : 1.0)
                    Button("Dismiss"){
                        dismissPopup = true
                        DispatchQueue.main.asyncAfter(deadline: .now() + 60 * 2) {
                            dismissPopup = false;
                        }
                    }
                    
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(Color(UIColor(hex: "#722F37")))
                .cornerRadius(16)
        
                Spacer()
            }
            .opacity(showPopup ? 1.0 : 0.0)
            .allowsHitTesting(showPopup)
            .onAppear {
                    withAnimation(.easeInOut(duration: 0.6).repeatForever(autoreverses: true)) {
                        blinking = true
                    }
                }
        }
        .padding(4)
        .focusable()
        .digitalCrownRotation(
            $value,
            from: minValue,
            through: maxValue,
            by: step,
            sensitivity: .low,
            isContinuous: false,
            isHapticFeedbackEnabled: true
        )
        .onChange(of: value) {
            updateValue()
        }
        .onChange(of: store.bacTarget) { oldValue, newValue in
            if(oldValue != newValue) {
                value = newValue
            }
        }
    }
}

#Preview {
    ContentView()
}

