using MessagePack;

namespace CheetahRelay.Tests
{
    [MessagePackObject]
    public class SomeStructure
    {
        [Key(0)] public long Age { get; set; }

        protected bool Equals(SomeStructure other)
        {
            return Age == other.Age;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            if (obj.GetType() != this.GetType()) return false;
            return Equals((SomeStructure) obj);
        }

        public override int GetHashCode()
        {
            return Age.GetHashCode();
        }
    }
}