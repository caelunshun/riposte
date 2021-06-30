local dume = require("dume")

local Text = require("widget/text")
local Button = require("widget/button")

local UiUtils = require("ui/utils")

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
    end
end
