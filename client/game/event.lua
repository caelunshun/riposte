-- An event bus.
local EventBus = {}

function EventBus:new()
    local o = {
        handlers = {}
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function EventBus:trigger(eventName, event)
    local handlers = self.handlers[eventName] or {}
    for i=1,#handlers do
        handlers[i](event)
    end
end

function EventBus:registerHandler(eventName, handlerFunction)
    local handlers = self.handlers[eventName] or {}
    self.handlers[eventName] = handlers
    handlers[#handlers + 1] = handlerFunction
end

return EventBus
