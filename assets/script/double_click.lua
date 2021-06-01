local handlers = {}

local delay = 0.5

function registerDoubleClickHandler(callback)
    handlers[#handlers + 1] = callback
end

local lastClickTime = 0

engine:registerEventHandler("onPosClicked", function(pos)
    local currentTime = os.time()

    local diff = os.difftime(currentTime, lastClickTime)
    if diff < delay then
        for _, handler in ipairs(handlers) do
            handler(pos)
        end
    end

    lastClickTime = currentTime
end)
