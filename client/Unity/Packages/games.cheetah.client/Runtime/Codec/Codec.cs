using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec
{
    /// <summary>
    /// Интерфейс сериализации объектов для взаимодействия между клиентом и сервером
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public interface Codec<T>
    {
        void Decode(ref CheetahBuffer buffer, ref T dest);
        void Encode(in T source, ref CheetahBuffer buffer);
    }
}