//
// Created by Caelum van Ispelen on 6/30/21.
//

#ifndef RIPOSTE_SLOT_MAP_H
#define RIPOSTE_SLOT_MAP_H

#include <cstdint>
#include <vector>
#include <limits>
#include <variant>

namespace rip {
    class InvalidIDException : public std::exception {
    public:
        const char *what() const noexcept override;
    };

    class TooManyItemsException : public std::exception {
    public:
        const char *what() const noexcept override;
    };

    // ID of an item in a slot_map.
    struct ID {
        uint16_t index;
        uint16_t version;

        ID() : index(0), version(0) {}

        ID(uint32_t encoded) : index(encoded), version(encoded >> 16) {}

        ID(uint16_t index, uint16_t version) : index(index), version(version) {}

        uint32_t encode() const {
            return static_cast<uint32_t>(index) | (static_cast<uint32_t>(version) << 16);
        }

        friend bool operator==(const ID &a, const ID &b) {
            return a.index == b.index && a.version == b.version;
        }

        friend bool operator!=(const ID &a, const ID &b) {
            return !(a == b);
        }

        template<typename H>
        friend H AbslHashValue(H h, const ID &id) {
            return H::combine(std::move(h), id.index, id.version);
        }
    };

    // Like a vector, except supports dynamically
    // and efficiently inserting new items. IDs are stable
    // and unique.
    template<typename T>
    class slot_map {
        std::vector<std::variant<T, uint8_t>> slots;
        std::vector<uint16_t> currentVersions;
        std::vector<uint16_t> freeIndices;
        size_t theSize = 0;

    public:
        struct Iterator {
            using iterator_category = std::forward_iterator_tag;
            using difference_type = std::ptrdiff_t;
            using value_type = T;
            using pointer = T*;
            using reference = T&;

            ID id;
            std::vector<std::variant<T, uint8_t>> *slots;

            Iterator(ID id, std::vector<std::variant<T, uint8_t>> *slots) : id(id), slots(slots) {}

            reference operator*() const {
                return std::get<0>((*slots)[id.index]);
            }

            pointer operator->() const {
                return &std::get<0>((*slots)[id.index]);
            }

            void advanceToOccupiedSlot() {
                auto &s = *slots;
                while (id.index < slots->size() && s[id.index].index() != 0) {
                    ++id.index;
                }
            }

            Iterator &operator++() {
                ++id.index;
                advanceToOccupiedSlot();

                if (id.index > slots->size()) id.index = slots->size();

                return *this;
            }

            Iterator operator++(int) {
                Iterator tmp = *this;
                ++(*this);
                return tmp;
            }

            friend bool operator==(const Iterator &a, const Iterator &b) {
                return a.id == b.id;
            }

            friend bool operator!=(const Iterator &a, const Iterator &b) {
                return !(a == b);
            }
        };

        struct ConstantIterator {
            using iterator_category = std::forward_iterator_tag;
            using difference_type = std::ptrdiff_t;
            using value_type = T;
            using pointer = const T*;
            using reference = const T&;

            ID id;
            const std::vector<std::variant<T, uint8_t>> *slots;

            ConstantIterator(ID id, const std::vector<std::variant<T, uint8_t>> *slots) : id(id), slots(slots) {}

            reference operator*() const {
                return std::get<0>((*slots)[id.index]);
            }

            pointer operator->() const {
                return &std::get<0>((*slots)[id.index]);
            }

            void advanceToOccupiedSlot() {
                while (id.index < slots->size() && (*slots)[id.index].index() != 0) {
                    ++id.index;
                }
            }

            ConstantIterator &operator++() {
                ++id.index;
                advanceToOccupiedSlot();

                if (id.index > slots->size()) id.index = slots->size();

                return *this;
            }

            ConstantIterator operator++(int) {
                ConstantIterator tmp = *this;
                ++(*this);
                return tmp;
            }

            friend bool operator==(const ConstantIterator &a, const ConstantIterator &b) {
                return a.id == b.id;
            }

            friend bool operator!=(const ConstantIterator &a, const ConstantIterator &b) {
                return !(a == b);
            }
        };

        slot_map() = default;

        // Determines whether the given value exists in the slotmap.
        bool contains(ID id) const {
            return id.index < slots.size()
                   && currentVersions[id.index] == id.version;
        }

        // Inserts a new item and returns its ID.
        ID insert(T value) {
            uint16_t index;
            if (!freeIndices.empty()) {
                index = freeIndices[freeIndices.size()-1];
                freeIndices.pop_back();
            } else {
                if (slots.size() + 1 > std::numeric_limits<uint16_t>::max()) {
                    throw TooManyItemsException();
                }
                slots.emplace_back((uint8_t) 0);
                currentVersions.emplace_back(0);
                index = slots.size() - 1;
            }

            uint16_t version = currentVersions[index];

            std::variant<T, uint8_t> slot(std::move(value));
            slots[index] = std::move(slot);

            ++theSize;

            return ID(index, version);
        }

        // Erases an item, allowing its index (but not its versioned ID)
        // to be recycled.
        void erase(ID id) {
            if (!contains(id)) return;

            slots[id.index] = (uint8_t) 0;

            // Invalidate the version.
            ++currentVersions[id.index];

            freeIndices.push_back(id.index);

            --theSize;
        }

        size_t size() const {
            return theSize;
        }

        T &operator[](ID id) {
            if (!contains(id)) {
                throw InvalidIDException();
            }
            return std::get<0>(slots[id.index]);
        }

        const T &operator[](ID id) const {
            if (!contains(id)) {
                throw InvalidIDException();
            }
            return std::get<0>(slots[id.index]);
        }

        Iterator begin() {
            auto it = Iterator(ID(0, 0), &slots);
            it.advanceToOccupiedSlot();
            return it;
        }

        Iterator end() {
            return Iterator(ID(slots.size(), 0), &slots);
        }

        ConstantIterator cbegin() const {
            auto it = ConstantIterator(ID(0, 0), &slots);
            it.advanceToOccupiedSlot();
            return it;
        }

        ConstantIterator cend() const {
            return ConstantIterator(ID(slots.size(), 0), &slots);
        }

        ConstantIterator begin() const {
            return cbegin();
        }

        ConstantIterator end() const {
            return cend();
        }
    };
}

#endif //RIPOSTE_SLOT_MAP_H
