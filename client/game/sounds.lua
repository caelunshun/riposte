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
    playSound("sound/event/turn_end", 0.1)
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

local function handleBuildTaskCompleted(game, event)
    if event.city.owner ~= game.thePlayer then return end
    if event.task == nil then return end

    local buildTask = event.task
    local sound
    local volume = 0.3
    if buildTask.kind.unit ~= nil then
        local unitKind = registry.unitKinds[buildTask.kind.unit.unitKindID]
        if unitKind.strength ~= 0 then
            sound = "sound/event/build_military"
            volume = 0.15
        end
    elseif buildTask.kind.building ~= nil then
        local buildingName = buildTask.kind.building.buildingName
        if buildingName == "Barracks" or buildingName == "Stable" then
            sound = "sound/event/build_military"
            volume = 0.15
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

local function handleCityCaptured(event)
    playSound("sound/event/city_capture", 0.1)
end

local function handleWarDeclared(event)
    playSound("sound/event/war_declared", 0.4)
end

local function handlePeaceDeclared(event)
    playSound("sound/event/peace_declared", 0.4)
end

local function handleBordersExpanded(event)
    playSound("sound/event/borders_expand", 0.3)
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
    game.eventBus:registerHandler("buildTaskFinished", function(event)
        handleBuildTaskCompleted(game, event)
    end)
    game.eventBus:registerHandler("cityCaptured", handleCityCaptured)
    game.eventBus:registerHandler("warDeclared", handleWarDeclared)
    game.eventBus:registerHandler("peaceDeclared", handlePeaceDeclared)
    game.eventBus:registerHandler("bordersExpanded", function(event)
        if event.city.owner == game.thePlayer then
            handleBordersExpanded(event)
        end
    end)
end

return registerSoundEvents
