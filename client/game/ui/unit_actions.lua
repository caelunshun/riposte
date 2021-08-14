local dume = require("dume")

local Text = require("widget/text")
local Button = require("widget/button")

local UiUtils = require("ui/utils")

local Vector = require("brinevector")

function getAdjacentTilePositions(tilePos)
    return {
        Vector(1, 0) + tilePos,
        Vector(1, 1) + tilePos,
        Vector(0, 1) + tilePos,
        Vector(-1, 1) + tilePos,
        Vector(-1, 0) + tilePos,
        Vector(-1, -1) + tilePos,
        Vector(0, -1) + tilePos,
        Vector(1, -1) + tilePos,
    }
end

-- Adds unit controls to a Flex row.
return function(row, units, game, hud)
    if #units == 0 then return end

    local kill = Button:new(Text:new("@size{20}{Kill}"), function()
        local unitSpecifier
        if #units == 1 then
            unitSpecifier = "your " .. units[1].kind.name
        else
            unitSpecifier = tostring(#units) .. " units"
        end
        UiUtils.openConfirmationPrompt("@size{15}{Are you sure you want to kill " .. unitSpecifier .. "?}", "Yes", "No", function()
            for _, unit in ipairs(units) do
                game.client:doUnitAction(unit, "Kill")
            end
        end)
    end)
    table.insert(kill.classes, "unitActionButton")
    row:addFixedChild(kill)

    if #units == 1 then
        local unit = units[1]

        if unit:hasCapability("found_city") and game:getCityAtPos(unit.pos) == nil then
            local foundCity = Button:new(Text:new("@size{20}{Found City}"), function()
                game.client:doUnitAction(unit, "FoundCity")
            end)
            table.insert(foundCity.classes, "unitActionButton")
            row:addFixedChild(foundCity)
        end

        if unit:hasCapability("bombard_city_defenses") then
            local city = nil
            for _, adjacentTilePos in ipairs(getAdjacentTilePositions(unit.pos)) do
                city = game:getCityAtPos(adjacentTilePos)
                if city ~= nil then break end
            end
            if city ~= nil and unit.owner:isAtWarWith(city.owner) and unit.movementLeft > 0 then
                local widget = Button:new(Text:new("@size{20}{Bombard City}"), function()
                    game.client:bombardCity(unit, city)
                    hud:clearSelection()
                end)
                table.insert(widget.classes, "unitActionButton")
                row:addFixedChild(widget)
            end
        end

        local workerCap = unit:getCapability("worker")
        if workerCap ~= nil then
            for _, possibleTask in ipairs(workerCap.possibleTasks) do
                local taskWidget = Button:new(Text:new("@size{20}{%taskName}", {taskName=possibleTask.name}), function()
                    game.client:setWorkerTask(unit, possibleTask)
                    hud:clearSelection()
                end)
                table.insert(taskWidget.classes, "unitActionButton")
                row:addFixedChild(taskWidget)
            end
        end
    end
end
