//
// Created by Caelum van Ispelen on 8/30/21.
//

#ifndef RIPOSTE_GRID_H
#define RIPOSTE_GRID_H

#include <vector>
#include <optional>
#include <string>
#include <cmath>

namespace rip::mapgen {
    class OutOfBoundsException : public std::exception {
        int width, height, x, y;
        std::string message;

    public:
        OutOfBoundsException(int width, int height, int x, int y);

        const char *what() const noexcept override;

    };

    // A grid of values of type T.
    //
    // The grid has a default value used for out-of-bounds
    // accesses.
    template<class T>
    class Grid {
        std::vector<T> theGrid;
        T defaultValue;
        uint32_t width;
        uint32_t height;

        std::optional<size_t> getIndex(int x, int y) const noexcept {
            if (x < width && y < height
                && x >= 0 && y >= 0) {
                return static_cast<size_t>(x + width * y);
            } else {
                return {};
            }
        }

        size_t getIndexOrThrow(int x, int y) const {
            const auto index = getIndex(x, y);
            if (index) {
                return *index;
            } else {
                throw OutOfBoundsException(width, height, x, y);
            }
        }

    public:
        // Constructs a grid with a width, height, and default value.
        //
        // All cells are filled with copies of the default value.
        Grid(uint32_t width, uint32_t height, T defaultValue) :
            width(width), height(height), defaultValue(std::move(defaultValue)),
                theGrid(std::vector<T>(width * height, this->defaultValue)) {}

        // Gets the value at the given integer coordinates, or
        // the default value if out of bounds.
        const T &get(int x, int y) const noexcept {
            const auto index = getIndex(x, y);
            if (index) {
                return theGrid[*index];
            } else {
                return defaultValue;
            }
        }

        // Mutably gets the value at the given integer coordinates.
        //
        // Throws OutOfBoundsException if out of bounds.
        T &get(int x, int y) {
            return theGrid[getIndexOrThrow(x, y)];
        }

        // Sets the value at the given integer coordinates.
        //
        // Throws OutOfBoundsException if out of bounds.
        void set(int x, int y, T value) {
            theGrid[getIndexOrThrow(x, y)] = std::move(value);
        }

        // Samples the grid at the given floating-point coordinates.
        const T &sample(float x, float y) const noexcept {
            return get(static_cast<int>(std::floor(x)), static_cast<int>(std::floor(y)));
        }

        // Resizes the grid to the given size.
        //
        // May shrink or grow along both axes.
        // If growing, new cells are filled with the default
        // value.
        void resizeTo(uint32_t newWidth, uint32_t newHeight) {
            std::vector<T> newGrid(newWidth * newHeight, defaultValue);

            for (int x = 0; x < std::min(width, newWidth); x++) {
                for (int y = 0; y < std::min(height, newHeight); y++) {
                    newGrid[x + y * newWidth] = get(x, y);
                }
            }

            theGrid = std::move(newGrid);
            width = newWidth;
            height = newHeight;
        }

        uint32_t getWidth() const noexcept {
            return width;
        }

        uint32_t getHeight() const noexcept {
            return height;
        }

        // "Stamps" the grid by applying an operator
        // to all cells on this grid overlapping the target region.
        //
        // The target region is defined by a width, height, and offset.
        template<typename Apply>
        void stamp(const Grid<T> &stamp, const int offsetX, const int offsetY,
                   const uint32_t targetWidth, const uint32_t targetHeight, Apply apply) noexcept {
            const int endX = std::min(offsetX + targetWidth, width - 1);
            const int endY = std::min(offsetY + targetHeight, height - 1);

            const float stepX = stamp.getWidth() / targetWidth;
            const float stepY = stamp.getHeight() / targetHeight;

            for (int x = offsetX; x < endX; x++) {
                for (int y = offsetY; y < endY; y++) {
                    const int dx = x - offsetX;
                    const int dy = y - offsetY;
                    const float fx = dx * stepX;
                    const float fy = dy * stepX;

                    const T &stampCell = stamp.sample(fx, fy);
                    T &ourCell = get(x, y);
                    apply(ourCell, stampCell);
                }
            }
        }
    };
}

#endif //RIPOSTE_GRID_H
