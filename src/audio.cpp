//
// Created by Caelum van Ispelen on 5/22/21.
//

#include <iostream>
#include "audio.h"

namespace rip {
    std::shared_ptr<Asset> AudioLoader::loadAsset(const std::string &id, const std::string &data) {
        auto handle = rodio_create_sound(reinterpret_cast<const uint8_t *>(data.data()), data.size());
        return std::make_shared<SoundAsset>(handle);
    }

    SoundAsset::SoundAsset(SoundHandle *handle) : handle(handle) {}

    InstanceHandle *AudioManager::playSound(const SoundAsset &sound, float volume) {
        return rodio_start_sound(rodio, sound.handle, volume);
    }

    void AudioManager::deleteSound(SoundId sound) {
        if (playingSounds.contains(sound)) {
            rodio_free_sound(playingSounds[sound].handle);
        }
        playingSounds.erase(sound);
    }

    AudioManager::AudioManager() {
        rodio = rodio_new();
    }

    void AudioManager::setAssets(std::shared_ptr<Assets> assets) {
        this->assets = std::move(assets);
    }

    SoundId AudioManager::playSound(const std::string &id, float volume) {
        const auto &handle = std::dynamic_pointer_cast<SoundAsset>(assets->get(id));
        auto *h = playSound(*handle, volume);
        const auto soundId = playingSounds.insert(Sound {
            .id = {},
            .handle = h,
        });
        playingSounds[soundId].id = soundId;
        return soundId;
    }

    void AudioManager::update() {
        // Check for completed sounds and delete them
        std::vector<SoundId> toDelete;
        for (auto &sound : playingSounds) {
            if (rodio_is_sound_done(sound.handle)) {
                toDelete.push_back(sound.id);
            }
        }

        for (const auto id : toDelete) {
            deleteSound(id);
        }
    }

    bool AudioManager::isSoundPlaying(SoundId sound) const {
        return playingSounds.contains(sound);
    }

    void AudioManager::stopSound(SoundId sound) {
        if (!playingSounds.contains(sound)) return;
        rodio_stop_sound(playingSounds[sound].handle);
        deleteSound(sound);
    }
}
