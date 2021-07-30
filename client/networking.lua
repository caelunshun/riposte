-- Asynchronous networking support based on callbacks.
-- Uses the protocol implemented in network.cpp.

local Networking = {}

function Networking:new()
    local o = {
        callbacks = {},
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

-- Connects to an endpoint and invokes `callback` once connected.
-- If successful, the first parameter contains a connection handle.
-- If an error occurs, the first parameter is nil and the second is an error message.
function Networking:connectAsync(ip, port, callback)
    local id = networkingConnectAsync(ip, port)
    self.callbacks[id] = callback
end

-- `message` is a string.
function Networking:sendMessage(connHandle, message)
    networkingSendAsync(connHandle, message)
end

-- Waits for a message and, when it arrives, invokes `callback`
-- with the payload (a string). If the payload is nil, then the second
-- parameter contains an error message.
function Networking:recvMessageAsync(connHandle, callback)
    local id = networkingRecvAsync(connHandle)
    self.callbacks[id] = callback
end

function Networking:tick()
    local completions = networkingPoll()
    for id, payload in pairs(completions) do
        self.callbacks[id](payload.contents, payload.error)
    end
end

return Networking
