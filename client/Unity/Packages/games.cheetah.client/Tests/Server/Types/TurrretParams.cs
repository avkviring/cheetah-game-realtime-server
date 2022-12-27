using Games.Cheetah.Client.Codec;

namespace Games.Cheetah.Client.Tests.Server.Types
{
    [GenerateCodec]
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
}