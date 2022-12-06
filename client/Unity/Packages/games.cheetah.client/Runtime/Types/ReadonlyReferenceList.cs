namespace Games.Cheetah.Client.Types
{
    public interface ReadonlyReferenceList<T> where T : struct
    {
        public int Count { get; }
        public ref T GetItem(int index);
    }
}