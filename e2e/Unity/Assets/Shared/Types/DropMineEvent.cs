using Cheetah.Matches.Relay.Codec;

namespace Shared.Types
{
    [GenerateCodec]
    public struct DropMineEvent
    {
        public int MineId;

        public override string ToString()
        {
            return $"MineId: {MineId}";
        }

        public bool Equals(DropMineEvent other)
        {
            return MineId == other.MineId;
        }

        public override bool Equals(object obj)
        {
            return obj is DropMineEvent other && Equals(other);
        }

        public override int GetHashCode()
        {
            return MineId;
        }
    }
}