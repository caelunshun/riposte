local function handleUnitMovement(game, event)
    if event.unit.owner ~= game.thePlayer then return end

    local targetTile = game:getTile(event.newPos)
    if targetTile.forested then
        playSound("sound/event/move_through_trees", 0.5)
    end

    playSound("sound/event/move", 0.375)
end

local function handleTurnChange(game)
    if game.turn == 0 then return end
    playSound("sound/event/turn_end", 0.25)
end

local currentCityHudSound

local function handleEnterCityHud()
    currentCityHudSound = playSound("sound/ambient/city1", 0.4)
end

local function handleCloseCityHud()
    if currentCityHudSound ~= nil then
        stopSound(currentCityHudSound)
        currentCityHudSound = nil
    end
end

local function handleBuildTaskCompleted(event)
    local buildTask = event.buildTask
    local sound
    local volume = 0.3
    if buildTask.kind.unit ~= nil then
        local unitKind = registry.unitKinds[buildTask.kind.unit.unitKindID]
        if unitKind.strength ~= 0 then
            sound = "sound/event/build_military"
        end
    elseif buildTask.kind.building ~= nil then
        local buildingName = buildTask.kind.building.buildingName
        if buildingName == "Barracks" then
            sound = "sound/event/build_military"
        elseif buildingName == "Granary" then
            sound = "sound/event/build_granary"
        elseif buildingName == "Library" then
            sound = "sound/event/build_library"
            volume = 0.6
        end
    end

    if sound ~= nil then
        playSound(sound, volume)
    end
end

local function registerSoundEvents(game)
    game.eventBus:registerHandler("unitMoved", function(event)
        handleUnitMovement(game, event)
    end)
    game.eventBus:registerHandler("turnChanged", function()
        handleTurnChange(game)
    end)
    game.eventBus:registerHandler("cityHudOpened", handleEnterCityHud)
    game.eventBus:registerHandler("cityHudClosed", handleCloseCityHud)
    game.eventBus:registerHandler("buildTaskCompleted", handleBuildTaskCompleted)
end

return registerSoundEvents
