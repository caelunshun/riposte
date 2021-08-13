local MenuMusic = {}

local volume = 1

function MenuMusic:new()
    local o = {
        sound = playSound("music/menu", volume),
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function MenuMusic:tick()
    if not isSoundPlaying(self.sound) then
        self.sound = playSound("music/menu", volume)
    end
end

function MenuMusic:close()
    stopSound(self.sound)
end

return MenuMusic
