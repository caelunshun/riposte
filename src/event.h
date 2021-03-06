//
// Created by Caelum van Ispelen on 5/26/21.
//

#ifndef RIPOSTE_EVENT_H
#define RIPOSTE_EVENT_H

#include <array>
#include <optional>
#include "era.h"

namespace rip {
    typedef std::array<uint8_t, 3> Color;

    static const Color colorBad = {193, 0, 22};
    static const Color colorGood = {68, 194, 113};
    static const Color colorTerrible = {60, 219, 192};

    struct Message {
        std::string text;
        Color color;

        Message(const std::string &text, const Color &color);
    };

    // An event in the game that triggers a response (HUD message, sound effect)
    class Event {
    public:
        virtual ~Event() = default;

        virtual std::optional<Message> getMessage() = 0;

        virtual std::optional<std::string> getAudioID(Era era) = 0;
    };

    class CombatEvent : public Event {
        bool won;
        std::string enemyAdjective;
        std::string ourUnitName;
        std::string enemyUnitName;

    public:
        CombatEvent(bool won, const std::string &enemyAdjective, const std::string &ourUnitName,
                    const std::string &enemyUnitName);

        std::optional<Message> getMessage() override;

        std::optional<std::string> getAudioID(Era era) override;
    };

    class CityCapturedEvent : public Event {
        std::string captured;
        std::string capturedByName;

    public:
        CityCapturedEvent(const std::string &captured, const std::string &capturedByName);

        std::optional<Message> getMessage() override;

        std::optional<std::string> getAudioID(Era era) override;
    };

    class WarDeclaredEvent : public Event {
        std::string declaredBy;
        std::string declaredOn;

    public:
        WarDeclaredEvent(const std::string &declaredBy, const std::string &declaredOn);

        ~WarDeclaredEvent() override = default;

        std::optional<Message> getMessage() override;

        std::optional<std::string> getAudioID(Era era) override;
    };

    class PlayerKilledEvent : public Event {
        std::string civName;

    public:
        PlayerKilledEvent(const std::string &civName);

        ~PlayerKilledEvent() override = default;

        std::optional<Message> getMessage() override;

        std::optional<std::string> getAudioID(Era era) override;
    };
}

#endif //RIPOSTE_EVENT_H
