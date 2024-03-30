import Foundation
// import RegexBuilder

struct ImageFileManager {
    let path: URL
    var count: Int = 0

    enum Error: Swift.Error {
        case existsIsNotDirectory
    }
    
    init(path: URL) throws {
        self.path = path

        var isDir: ObjCBool = true
        if !FileManager.default.fileExists(atPath: path.relativeString, isDirectory: &isDir) {
            try FileManager.default.createDirectory(at: path, withIntermediateDirectories: true)
        } else {
            if !isDir.boolValue {
                throw Self.Error.existsIsNotDirectory
            }
        }

        let items = try FileManager.default.contentsOfDirectory(atPath: path.relativeString)
        let fileRegex = ##/image(\d+).jpg/##
        for item in items {
            if let result = try? fileRegex.wholeMatch(in: item) {
                let res = Int(result.1)!
                if res > self.count {
                    self.count = res
                }
            }
        }

        self.count += 1
    }

    mutating func nextPath() -> URL {
        defer { self.count += 1 }
        return self.path.appendingPathComponent("image\(self.count).jpg")
    }

    var previousPath: URL {
        return self.path.appendingPathComponent("image\(self.count - 1).jpg")
    }
}

