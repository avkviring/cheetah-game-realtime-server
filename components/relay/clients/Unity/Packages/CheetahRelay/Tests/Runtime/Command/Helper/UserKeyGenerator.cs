namespace CheetahRelay.Tests
{
    public static class UserKeyGenerator
    {
        public static CheetahBuffer Private = GetPrivateKey();
        private static ushort nextPublic = 1;

        private static CheetahBuffer GetPrivateKey()
        {
            var key = new CheetahBuffer();
            for (var i = 0; i < 32; i++)
            {
                key.Add(5);
            }

            return key;
        }

        public static ushort NextPublic()
        {
            return nextPublic++;
        }
    }
}