import Foundation

enum LogType {
    case verbose
    case info
    case warning
    case error
}

extension LogType: CustomStringConvertible {
    var description: String {
        switch (self) {
            case .info: "INFO"
            case .error: "ERROR"
            case .warning: "WARN"
            case .verbose: "VERBOSE"
        }
    }
    
    var level: Int {
        switch (self) {
            case .verbose: 0
            case .info: 1
            case .warning: 2
            case .error: 3
        }
    }
}

var logging: [(type: LogType, date: Date, message: String)] = []

let minLoggingLevel: Int = LogType.info.level

func log(_ type: LogType, _ text: String) {
    if (type.level < minLoggingLevel) { return }

    let date = Date()
    print("[\(type) \(date)] \(text)")
    logging.append((type: type, date: date, message: text))
}

