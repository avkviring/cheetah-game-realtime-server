using System;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Types;

namespace Tests.Types
{
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

    public class DropMineEventCodec : Codec<DropMineEvent>
    {
        public void Decode(ref CheetahBuffer buffer, ref DropMineEvent dest)
        {
            dest.MineId = PrimitiveReaders.ReadInt(ref buffer);
        }

        public void Encode(ref DropMineEvent source, ref CheetahBuffer buffer)
        {
            PrimitiveWriters.Write(source.MineId, ref buffer);
        }

        public string Dump(ref CheetahBuffer buffer)
        {
            throw new NotImplementedException();
        }
    }
}