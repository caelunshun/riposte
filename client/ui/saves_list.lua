local SavesListWindow = {}

local Vector = require("brinevector")
local dume = require("dume")

local Flex = require("widget/flex")
local Table = require("widget/table")
local Text = require("widget/text")
local Divider = require("widget/divider")
local Spacer = require("widget/spacer")
local Button = require("widget/button")
local Padding = require("widget/padding")
local Empty = require("widget/empty")

local UiUtils = require("ui/utils")

function SavesListWindow:new(category, onSelect)
    local o = {
        category = category,
        onSelect = onSelect,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function SavesListWindow:rebuild()
    local root = Flex:column(20)
    root:setCrossAlign(dume.Align.Center)

    root:addFixedChild(Text:new("@size{24}{Available Game Saves (%category)}", {
        category = self.category
    }))
    root:addFixedChild(Divider:new(1))

    local rows = {}
    rows[1] = {
        name = Text:new("Name"),
        turn = Text:new("Turn"),
        load = Empty:new(),
        delete = Empty:new()
    }

    local saves = getAllSaves(self.category)
    table.sort(saves, function(a, b)
        return a.turn > b.turn
    end)
    for _, saveFile in ipairs(saves) do
        rows[#rows + 1] = {
            name = Text:new("%name", {
                name = saveFile.name
            }),
            turn = Text:new("%turn", {
                turn = tostring(saveFile.turn),
            }),
            load = Button:new(Padding:new(Text:new("Load"), 10), function()
                self.onSelect(saveFile)
                self:close()
            end),
            delete = Button:new(Padding:new(Text:new("Delete"), 10), function()
                os.remove(saveFile.path)
                self:rebuild()
            end),
        }
    end

    root:addFixedChild(Table:new({
        "name",
        "turn",
        "load",
        "delete",
    }, rows))

    root:addFixedChild(Button:new(Padding:new(Text:new("@size{20}{Back}"), 10), function()
        self.onSelect(nil)
        self:close()
    end))

    local root = UiUtils.createWindowContainer(root)
    ui:createWindow("savesList", dume.FillScreen, root, 1)
end

function SavesListWindow:close()
    ui:deleteWindow("savesList")
end

return SavesListWindow
