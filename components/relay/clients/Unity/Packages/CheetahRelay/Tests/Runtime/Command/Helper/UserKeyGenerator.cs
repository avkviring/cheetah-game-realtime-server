namespace CheetahRelay.Tests
{
    public static class UserKeyGenerator
    {
        public static CheetahBuffer PrivateKey;

        private static ushort nextPublic = 1;

        private static void MakePrivateKey()
        {
            PrivateKey.Clear();
            PrivateKey.Add((byte) nextPublic);
            for (var i = 1; i < 32; i++)
            {
                PrivateKey.Add(5);
            }
        }

        public static ushort GetNextUserId()
        {
            nextPublic++;
            MakePrivateKey();
            return nextPublic;
        }
    }
}