-- Manages a list of coroutines scheduled to be woken after
-- a specified period of time.
local Scheduler = {}

function Scheduler:new()
    local o = {
        tasks = {},
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Scheduler:addTask(task, wakeAfter)
    self.tasks[#self.tasks + 1] = {
        thread = task,
        wakeAt = time + wakeAfter,
    }
end

function Scheduler:tick(time)
    local toRemove = {}
    for i=1,#self.tasks do
        local task = self.tasks[i]
        if time <= task.wakeAt then
            callSafe(task.thread)
            toRemove[#toRemove + 1] = i
        end
    end

    for j, i in ipairs(toRemove) do
        table.remove(self.tasks, i - (j - 1))
    end
end

-- Waits for the given number of seconds.
-- Must be called from within a coroutine.
function sleepFor(seconds)
    scheduler:addTask(coroutine.running(), seconds)
end

return Scheduler
