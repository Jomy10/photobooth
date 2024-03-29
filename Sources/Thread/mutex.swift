import Foundation

public final class Mutex<T> {
    private final var mutex: pthread_mutex_t = pthread_mutex_t()
    private final var value: T

    public enum Error: Swift.Error {
        case systemTemporarilyLacksResources
        case invalidAttributes
        case noMemory

        case deadLock
        case invalidMutex

        case threadDoesNotHoldThisMutex

        case unspecifiedError(Int32)
    }

    public init(_ t: T) throws {
        self.value = t
	      
	    var attr: pthread_mutexattr_t = pthread_mutexattr_t()
	    pthread_mutexattr_init(&attr)
	    pthread_mutexattr_settype(&attr, Int32(PTHREAD_MUTEX_RECURSIVE))

        let err = pthread_mutex_init(&self.mutex, &attr)
        pthread_mutexattr_destroy(&attr)

        switch err {
        case 0:
            break
            // succes
        case EAGAIN:
            throw Self.Error.systemTemporarilyLacksResources
        case EINVAL:
            throw Self.Error.invalidAttributes
        case ENOMEM:
            throw Self.Error.noMemory
        default:
            throw Self.Error.unspecifiedError(err)
        }
    }

    public final func lock() throws {
        let ret = pthread_mutex_lock(&self.mutex)
        switch ret {
        case 0:
            break
        case EDEADLK:
            throw Self.Error.deadLock
        case EINVAL:
            throw Self.Error.invalidMutex
        default:
            throw Self.Error.unspecifiedError(ret)
        }
    }

    public final func unlock() throws {
        let ret = pthread_mutex_unlock(&self.mutex)
        switch ret {
        case 0:
            break
        case EPERM:
            throw Self.Error.threadDoesNotHoldThisMutex
        case EINVAL:
            throw Self.Error.invalidMutex
        default:
            throw Self.Error.unspecifiedError(ret)
        }
    }

    @discardableResult
    public final func locked<Ret>(_ block: (T) throws -> (Ret)) throws -> Ret {
        try self.lock()

        defer {
            try? self.unlock()
        }

        let ret: Ret = try block(self.value)
        return ret
    }

    @discardableResult
    public final func lockedMut<Ret>(_ block: (inout T) throws -> (Ret)) throws -> Ret {
        try self.lock()

        defer {
            do {
                try self.unlock()
            } catch let err {
                print("Mutex unlock error: \(err)")
            }
        }

        let ret: Ret = try block(&self.value)
        return ret
    }

    public var unsafeValue: T {
        set { self.value = newValue }
        get { self.value }
    }

    deinit {
        pthread_mutex_destroy(&self.mutex)
    }
}

