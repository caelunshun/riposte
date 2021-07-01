local MusicPlayer = {}

local eras = {
    Ancient = "ancient",
    Classical = "classical",
    Medieval = "medieval",
    Renaissance = "renaissance",
    Industrial = "industrial",
    Modern = "modern",
    Future = "future",
}

function MusicPlayer:new(game)
    local o = { game = game }
    setmetatable(o, self)
    self.__index = self

    o:loadEraMusics()

    game.eventBus:registerHandler("eraChanged", function()
        o:selectNewSong()
    end)

    return o
end

function MusicPlayer:loadEraMusics()
    self.music = {}

    for eraName, eraID in pairs(eras) do
        local musics = getAssetIDsWithPrefix("music/" .. eraID)
        if #musics <= 1 then
            error("too few soundtracks for era " .. eraName)
        end
        self.music[eraName] = musics

        print("[lua] Loaded " .. #musics .. " soundtracks for era " .. eraName)
    end
end

function MusicPlayer:selectNewSong()
    math.randomseed(os.time())
    local newSong = nil
    while newSong == nil do
        local candidates = self.music[self.game.era]
        local candidate = candidates[math.random(1, #candidates)]

        -- Don't play the same song twice.
        if candidate ~= self.previousSong then
            newSong = candidate
        end
    end

    if self.currentSound ~= nil then
        stopSound(self.currentSound)
    end

    print("Soundtrack set to " .. newSong)

    self.currentSound = playSound(newSong)
    self.previousSong = newSong
end

function MusicPlayer:tick()
    if self.currentSound == nil then
        self:selectNewSong()
    end

    if not isSoundPlaying(self.currentSound) then
        self:selectNewSong()
    end
end

return MusicPlayer
