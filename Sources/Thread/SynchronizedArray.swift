import Foundation

/// Lock-free, blocking, thread-safe array implementation
public final class SynchronizedArray<Element> {
    private let queue = DispatchQueue(label: "be.jonaseveraert.Threads.SynchronizedArray")
    private var array: Array<Element>

    public init() {
        self.array = Array()
    }

    public convenience init(_ initArray: Array<Element>) {
        self.init()
        self.array = initArray
    }
}

extension SynchronizedArray: Collection, RandomAccessCollection, MutableCollection {
    public typealias Index = Array<Element>.Index
    public typealias Element = Array<Element>.Element
    
    public var startIndex: Index {
        var startIndex: Index? = nil
        self.queue.sync { startIndex = self.array.startIndex }
        return startIndex!
    }

    public var endIndex: Index {
        var endIndex: Index? = nil
        self.queue.sync { endIndex = self.array.endIndex }
        return endIndex!
    }

    public subscript(index: Index) -> Element {
        get {
            var element: Element? = nil
            self.queue.sync { element = self.array[index] }
            return element!
        }
        set {
            self.queue.sync { self.array[index] = newValue }
        }
    }

    public func index(after i: Index) -> Index {
        var index: Index? = nil
        self.queue.sync { index = self.array.index(after: i) }
        return index!
    }
}

extension SynchronizedArray: RangeReplaceableCollection {
    public func replaceSubrange<C>(_ subrange: Range<Array<Element>.Index>, with newElements: C)
    where C : Collection, Array<Element>.Element == C.Element {
        self.queue.sync { 
            self.array.replaceSubrange(subrange, with: newElements)
        }
    }
}

// Specific re-implementations of standard functions to make them synchronous
extension SynchronizedArray {
    public func forEachSync(_ cb: (Element) -> Void) {
        self.queue.sync {
            self.array.forEach(cb)
        }
    }
}

