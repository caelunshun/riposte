//
// Created by Caelum van Ispelen on 5/26/21.
//

#include "event.h"
#include "ripmath.h"

namespace rip {
    CombatEvent::CombatEvent(bool won, const std::string &enemyAdjective, const std::string &ourUnitName,
                             const std::string &enemyUnitName) : won(won), enemyAdjective(enemyAdjective),
                                                                  ourUnitName(ourUnitName),
                                                                  enemyUnitName(enemyUnitName) {}

    std::optional<Message> CombatEvent::getMessage() {
       if (won) {
           return Message(
                   "Your " + ourUnitName + " has defeated " + article(enemyAdjective) + " " + enemyAdjective + " " + enemyUnitName + "!",
                   colorGood
                   );
       } else {
           return Message(
                   "Your " + ourUnitName + " has died fighting " + article(enemyAdjective) + " " + enemyAdjective + " " +  enemyUnitName + "!",
                   colorBad
                   );
       }
    }

    std::optional<std::string> CombatEvent::getAudioID(Era era) {
        if (won) {
            return "sound/event/combat_victory";
        } else {
            return "sound/event/combat_defeat";
        }
    }

    Message::Message(const std::string &text, const Color &color) : text(text), color(color) {}

    CityCapturedEvent::CityCapturedEvent(const std::string &captured, const std::string &capturedByName) : captured(
            captured), capturedByName(capturedByName) {}

    std::optional<Message> CityCapturedEvent::getMessage() {
        return Message(
                captured + " has been captured by " + capturedByName + "!",
                colorBad
                );
    }

    std::optional<std::string> CityCapturedEvent::getAudioID(Era era) {
        return "sound/event/city_capture";
    }
}
