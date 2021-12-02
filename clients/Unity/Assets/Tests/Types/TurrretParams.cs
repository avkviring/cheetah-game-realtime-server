using System;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Types;

namespace Tests.Types
{
    public struct TurretsParamsStructure
    {
        public double Speed;
        public double Damage;

        public bool Equals(TurretsParamsStructure other)
        {
            return Speed.Equals(other.Speed) && Damage.Equals(other.Damage);
        }

        public override bool Equals(object obj)
        {
            return obj is TurretsParamsStructure other && Equals(other);
        }

        public override int GetHashCode()
        {
            unchecked
            {
                return (Speed.GetHashCode() * 397) ^ Damage.GetHashCode();
            }
        }

        public override string ToString()
        {
            return $"Speed: {Speed}, Damage: {Damage}";
        }
    }

    public class TurretsParamsStructureCodec : Codec<TurretsParamsStructure>
    {
        public void Decode(ref CheetahBuffer buffer, ref TurretsParamsStructure dest)
        {
            dest.Damage = PrimitiveReaders.ReadDouble(ref buffer);
            dest.Speed = PrimitiveReaders.ReadDouble(ref buffer);
        }

        public void Encode(ref TurretsParamsStructure source, ref CheetahBuffer buffer)
        {
            PrimitiveWriters.Write(source.Damage, ref buffer);
            PrimitiveWriters.Write(source.Speed, ref buffer);
        }

        public string Dump(ref CheetahBuffer buffer)
        {
            throw new NotImplementedException();
        }
    }
}