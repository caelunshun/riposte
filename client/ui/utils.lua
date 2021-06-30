local UiUtils = {}

local dume = require("dume")
local Vector = require("brinevector")

local Text = require("widget/text")
local Container = require("widget/container")
local Button = require("widget/button")
local Flex = require("widget/flex")
local Padding = require("widget/padding")
local Center = require("widget/center")

function UiUtils.openConfirmationPrompt(title, confirmationText, negationText, onconfirm)
    local root = Flex:column(10)

    local titleText = Text:new(title)
    root:addFixedChild(titleText)

    local options = Flex:row()
    options:setMainAlign(dume.Align.Center)

    local confirmation = Button:new(Center:new(Text:new(confirmationText)), function()
        ui:deleteWindow("confirmation")
        onconfirm()
    end)
    table.insert(confirmation.classes, "confirmationButton")
    options:addFlexChild(confirmation, 1)
    local negation = Button:new(Center:new(Text:new(negationText)), function()
        ui:deleteWindow("confirmation")
    end)
    table.insert(negation.classes, "confirmationButton")
    options:addFlexChild(negation, 1)

    root:addFixedChild(options)

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(300, 100)
    ui:createWindow("confirmation", Vector(cv:getWidth() - size.x - 10, 10), size, container)
end

return UiUtils