namespace Games.Cheetah.Client.Types
{
    public struct StringReference
    {
        internal string value;

        public string GetString()
        {
            return value;
        }

        public StringReference(string s)
        {
            value = s;
        }
    }
}