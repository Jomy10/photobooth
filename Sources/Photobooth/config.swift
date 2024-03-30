import Foundation
import Yams

struct PhotoboothConfig: Codable {
    var loggingPath: String = "photobooth_log.txt"
    var imagePath: String = "images"
    var doneSentences: [String] = [
        "All done!",
        "You look great!",
        "Come closer again",
        "Looking good 😎",
        "Curious to see the result?"
    ]
    var bgColor: UInt32 = 0xFF32a8a8

    init() {}

    static func read(file: String) throws -> Self? {
        let decoder = YAMLDecoder()
        if !FileManager.default.fileExists(atPath: file) {
            return nil
        }
        return try decoder.decode(Self.self, from: String(contentsOfFile: file))
    }

    enum CodingKeys: String, CodingKey {
        case loggingPath
        case imagePath
        case doneSentences
        case bgColor
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: Self.CodingKeys.self)
        if let loggingPath = try container.decodeIfPresent(String.self, forKey: .loggingPath) {
            self.loggingPath = loggingPath
        }
        if let imagePath = try container.decodeIfPresent(String.self, forKey: .imagePath) {
            self.imagePath = imagePath
        }
        if let doneSentences = try container.decodeIfPresent([String].self, forKey: .doneSentences) {
            self.doneSentences = doneSentences.map { sentence in sentence.replacingOccurrences(of: "\\n", with: "\n") }
        }
        if let bgColor = try container.decodeIfPresent(UInt32.self, forKey: .bgColor) {
            self.bgColor = bgColor
        }
        log(.info, "Configuration read: \(self)")
    }
}

