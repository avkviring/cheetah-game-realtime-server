namespace CheetahRelay.Tests
{
    public class AuthHelper
    {
        private static ulong GlobalRoomId = 0;
        public ushort UserId = 0;
        public CheetahBuffer PrivateKey;

        public ulong RoomId;

        public AuthHelper()
        {
            GlobalRoomId++;
            RoomId = GlobalRoomId;
            UserId = 0;
        }

        public void MakeUserPrivateKey()
        {
            PrivateKey.Clear();
            PrivateKey.Add((byte) RoomId);
            PrivateKey.Add((byte) UserId);
            for (var i = 1; i < 31; i++)
            {
                PrivateKey.Add(5);
            }
        }

        public ushort GetNextUserId()
        {
            UserId++;
            MakeUserPrivateKey();
            return UserId;
        }
    }
}