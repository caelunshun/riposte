local UiUtils = {}

local dume = require("dume")
local Vector = require("brinevector")

local Text = require("widget/text")
local Container = require("widget/container")
local Button = require("widget/button")
local Flex = require("widget/flex")
local Padding = require("widget/padding")
local Center = require("widget/center")

function UiUtils.createWindowContainer(child)
    local container = Container:new(Padding:new(child, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")
    return container
end

function UiUtils.openConfirmationPrompt(title, confirmationText, negationText, onconfirm)
    local root = Flex:column(10)

    local titleText = Text:new(title)
    root:addFixedChild(titleText)

    local options = Flex:row()
    options:setMainAlign(dume.Align.Center)

    local confirmation = Button:new(Text:new(confirmationText), function()
        ui:deleteWindow("confirmation")
        onconfirm()
    end)
    table.insert(confirmation.classes, "confirmationButton")
    options:addFlexChild(confirmation, 1)
    local negation = Button:new(Text:new(negationText), function()
        ui:deleteWindow("confirmation")
    end)
    table.insert(negation.classes, "confirmationButton")
    options:addFlexChild(negation, 1)

    root:addFixedChild(options)

    local container = UiUtils.createWindowContainer(root)

    local size = Vector(300, 100)
    ui:createWindow("confirmation", function(screenSize)
        return {
            pos = Vector(screenSize.x - size.x - 10, 10),
            size = size,
        }
    end, container, 10)
end

function maybeInfinity(x)
    if x == math.huge then
        return "∞"
    else return tostring(x)
    end
end

function maybePlural(noun, amount)
    if amount ~= 1 then
        return noun .. "s"
    else
        return noun
    end
end

function UiUtils.centerWindow(size)
    return function(screenSize)
        return {
            pos = Vector(screenSize.x / 2 - size.x / 2, screenSize.y / 2 - size.y / 2),
            size = size,
        }
    end
end

return UiUtils
