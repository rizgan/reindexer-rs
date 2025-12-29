# FindLevelDB.cmake
# Finds LevelDB library

set(LevelDB_FOUND TRUE)
set(LevelDB_INCLUDE_DIR "${LEVELDB_INCLUDE_DIR}" CACHE PATH "LevelDB include directory")
set(LevelDB_LIBRARY "${LEVELDB_LIBRARY}" CACHE FILEPATH "LevelDB library")

mark_as_advanced(LevelDB_INCLUDE_DIR LevelDB_LIBRARY)
