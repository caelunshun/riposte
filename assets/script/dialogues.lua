Status = {
  Furious = 0,
  Annoyed = 1,
  Cautious = 2,
  Pleased = 3,
  Friendly = 4,
}

function getGreeting(greetingPlayer, greetedPlayer)
  local r = {}

  local leader = greetingPlayer:getLeader()
  -- local status = greetingPlayer:getStatusWith(greetedPlayer)
  local status = Status.Furious;

  if status >= Status.Pleased then
    r[#r + 1] = "What can we do for you?"
    r[#r + 1] = "How can we help?"
    r[#r + 1] = "What would you like from us?"

    if status == Status.Friendly then
      r[#r + 1] = string.format("Good evening, %s.", greetedPlayer:getName())
    end
  end

  if status >= Status.Cautious then
    r[#r + 1] = "Yes?"
    r[#r + 1] = "What would you like?"

    if leader.aggressive >= 5 then
      r[#r + 1] = "I live by the sword."
    end
  end

  if status <= Status.Annoyed then
    if leader.aggressive >= 5 then
      r[#r + 1] = "I would beat you, but I'd infect my hands."
    end

    r[#r + 1] = "What do you want?"
    r[#r + 1] = "What do you want now?"
    r[#r + 1] = "What now?"

    if leader.paranoia >= 6 then
      r[#r + 1] = "Stop wasting my time."
    end

    if status == Status.Furious then
      r[#r + 1] = "The sight of you infects my eyes."
      r[#r + 1] = "Get out or we'll shoot you."
    end
  end

  local index = math.random(1, #r)
  return r[index]
end

function getWarDeclaration(greetingPlayer, greetedPlayer)
  local r = {}

  r[#r + 1] = "Our troops will be conducting some training exercises. On your land."
  r[#r + 1] = "I've had enough of you, " .. greetedPlayer:getName() .. "."
  r[#r + 1] = "Prepare to die."
  r[#r + 1] = "Say goodbye to your cities, " .. greetedPlayer:getName() .. "."

  if greetingPlayer:getLeader().aggressive >= 6 then
    r[#r + 1] = "You are but an ant to us. We will stampede you on the path to domination."
  end

  local index = math.random(1, #r)
  return r[index] .. string.format(" (%s declares war!)", greetingPlayer:getName())
end

if DialogueWindow == nil then
  DialogueWindow = {}
end

local DialogueState = {
  Main = 0,
  DeclareWar = 1,
}

local currentDialogue = nil

function DialogueWindow.paintOptions(self, ui)
  if self.state == DialogueState.Main then
    if not game:getThePlayer():isAtWarWith(self.withPlayer) and ui:buttonLabel("Your head would look good on the end of a pole. (WAR)") then
      self.thePlayer:declareWarOn(self.withPlayer)
      self.close = true
    end

    if ui:buttonLabel("Farewell (exit)") then
      self.close = true
    end
  elseif self.state == DialogueState.DeclareWar then
    if ui:buttonLabel("So be it.") then
      self.close = true
    end
  end
end

local numDialogues = 0

function DialogueWindow.paint(self, ui)
  -- center window on screen
  local windowSize = game:getCursor():getWindowSize()
  local size = { x = 500, y = 600}
  ui:beginWindow(string.format("dialogueWindow%d", numDialogues), windowSize.x / 2 - size.x / 2, windowSize.y / 2 - size.y / 2, size.x, size.y)

  ui:layoutDynamic(50, 1)

  local text = self.withPlayer:getName() .. " of the " .. self.withPlayer:getCiv().name -- .. " (" .. self.withPlayer:getStatusWith(self.thePlayer) .. ")"
  ui:label(text)

  ui:layoutDynamic(100, 1)
  ui:spacing(1)

  ui:layoutDynamic(100, 1)
  ui:labelWrap(self.text)

  -- Options
  ui:layoutDynamic(50, 1)
  self:paintOptions(ui)

  ui:endWindow()
end

function DialogueWindow.shouldClose(self)
  return self.close
end

function DialogueWindow.new(self, state, thePlayer, withPlayer)
  if currentDialogue ~= nil then
    currentDialogue.close = true
  end

  numDialogues = numDialogues + 1
  o = { state = state, close = false, thePlayer = thePlayer, withPlayer = withPlayer }

  if state == DialogueState.Main then
    o.text = getGreeting(withPlayer, thePlayer)
  elseif state == DialogueState.DeclareWar then
    o.text = getWarDeclaration(withPlayer, thePlayer)
  end

  setmetatable(o, self)
  self.__index = self
  if (o.paint == nil) then error("foo") end

  currentDialogue = o

  return o
end

engine:registerEventHandler("onWarDeclared", function(declaringPlayer, declaredPlayer)
  if declaringPlayer:hasAI() and not declaredPlayer:hasAI() then
    hud:openWindow(DialogueWindow:new(DialogueState.DeclareWar, declaredPlayer, declaringPlayer))
  end
  end)

engine:registerEventHandler("onDialogueOpened", function(withPlayer)
  hud:openWindow(DialogueWindow:new(DialogueState.Main, game:getThePlayer(), withPlayer))
end)
