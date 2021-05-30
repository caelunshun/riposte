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
                captured + " has been captured by the " + capturedByName + "!",
                colorBad
                );
    }

    std::optional<std::string> CityCapturedEvent::getAudioID(Era era) {
        return "sound/event/city_capture";
    }

    WarDeclaredEvent::WarDeclaredEvent(const std::string &declaredBy, const std::string &declaredOn) : declaredBy(
            declaredBy), declaredOn(declaredOn) {}

    std::optional<Message> WarDeclaredEvent::getMessage() {
        return Message(
                declaredBy + " has declared war on " + declaredOn + "!",
                colorBad
                );
    }

    std::optional<std::string> WarDeclaredEvent::getAudioID(Era era) {
        return "sound/event/combat_defeat";
    }

    std::optional<Message> PlayerKilledEvent::getMessage() {
        return Message(
                "The " + civName + " has been destroyed!",
                colorTerrible
                );
    }

    std::optional<std::string> PlayerKilledEvent::getAudioID(Era era) {
        return std::optional<std::string>();
    }

    PlayerKilledEvent::PlayerKilledEvent(const std::string &civName) : civName(civName) {}
}
