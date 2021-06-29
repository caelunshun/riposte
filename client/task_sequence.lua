-- A sequence of coroutines that will execute sequentially.
local TaskSequence = {}

function TaskSequence:new()
    local o = {
        tasks = {},
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function TaskSequence:enqueue(task)
    table.insert(self.tasks, task)
    if #self.tasks == 1 then
        callSafe(task)
        self:tick()
    end
end

function TaskSequence:tick()
    while #self.tasks > 0 do
        if coroutine.status(self.tasks[1]) == "dead" then
            table.remove(self.tasks, 1)

            if #self.tasks > 0 then
                callSafe(self.tasks[1])
            end
        else
            return
        end
    end
end

return TaskSequence
