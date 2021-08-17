-- Implements combat animation.
--
-- See the protocol file for information on how the client handles combat.
--
-- The `currentCombatEvent` field on `Game` is set to an instance of this class
-- if a combat animation is currently being displayed.
local CombatEvent = {}

local timeBetweenRounds = 0.3

-- Creates a CombatEvent from the CombatEvent packet.
function CombatEvent:new(game, packet)
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

    if time - self.previousRoundTime >= timeBetweenRounds then
        self.previousRoundTime = time

        -- update attacker and defender healths
        self.attacker.health = self.rounds[self.nextRound].attackerHealth
        self.defender.health = self.rounds[self.nextRound].defenderHealth

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
