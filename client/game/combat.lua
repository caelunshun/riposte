-- Implements combat animation.
--
-- See the protocol file for information on how the client handles combat.
--
-- The `currentCombatEvent` field on `Game` is set to an instance of this class
-- if a combat animation is currently being displayed.
local CombatEvent = {}

local function getTimePerRound(numRounds)
    local desiredCombatTime = 4
    return desiredCombatTime / numRounds
end

-- Creates a CombatEvent from the CombatEvent packet.
function CombatEvent:new(game, packet)
    print(inspect(packet))
    local o = {
        game = game,
        defender = game.units[packet.defenderID],
        attacker = game.units[packet.attackerID],
        rounds = packet.rounds,
        previousRoundTime = time,
        nextRound = 1,
        finished = false,
        attackerWon = packet.attackerWon,
        numCollateralTargets = packet.numCollateralTargets,
        timePerRound = getTimePerRound(#packet.rounds),
    }
    o.prevRound = {
        attackerHealth = o.attacker.health,
        defenderHealth = o.defender.health,
    }
    setmetatable(o, self)
    self.__index = self

    o.defender.isInCombat = true
    o.attacker.isInCombat = true

    return o
end

-- Advances the combat animation, updating the healths of involved units.
function CombatEvent:tick()
    if self.finished then return end

    if self.rounds[self.nextRound] == nil then
        self:finish()
        return
    end

    -- interpolate health values
    local elapsed = math.clamp((time - self.previousRoundTime) / self.timePerRound, 0, 1)
    self.attacker.health = self.prevRound.attackerHealth * (1 - elapsed) + self.rounds[self.nextRound].attackerHealth * elapsed
    self.defender.health = self.prevRound.defenderHealth * (1 - elapsed) + self.rounds[self.nextRound].defenderHealth * elapsed

    if time - self.previousRoundTime >= self.timePerRound then
        self.previousRoundTime = time

        self.prevRound = self.rounds[self.nextRound]
        self.nextRound = self.nextRound + 1

        if self.nextRound > #self.rounds then
            self:finish()
        end
    end
end

function CombatEvent:finish()
    self.attacker.isInCombat = false
    self.defender.isInCombat = false
    self.game:clearCombatEvent()

    self.game.messages:onCombatFinished(self.game, self)

    print("finished combat")
end

return CombatEvent
