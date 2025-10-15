import { FormEvent, useCallback, useMemo, useState } from "react";

const API_BASE = import.meta.env.VITE_APATO_API ?? "http://localhost:8080";

interface Watchlist {
  id: number;
  location_name: string;
  target_yield: number | null;
  target_size_min: number | null;
  target_size_max: number | null;
  created_at: string;
  updated_at: string;
}

interface Apartment {
  id: number;
  location_name: string | null;
  size: number | null;
  price: number | null;
  rent: number | null;
  estimated_yield: number | null;
  url: string | null;
  created_at: string;
}

type ApiResponse<T> = { data: T };

type WatchlistsResponse = ApiResponse<{ watchlists: Watchlist[] }>;

type ApartmentsResponse = ApiResponse<{ apartments: Apartment[] }>;

type Status = { message: string; tone?: "success" | "error" } | null;

function formatNumber(value: number | null | undefined, digits = 2) {
  if (value === null || value === undefined) return "-";
  return Number(value).toFixed(digits);
}

function formatCurrency(value: number | null | undefined) {
  if (value === null || value === undefined) return "-";
  return new Intl.NumberFormat("en-US", { style: "currency", currency: "EUR" }).format(
    value
  );
}

function formatDate(value: string) {
  return new Date(value).toLocaleString();
}

const App = () => {
  const [chatId, setChatId] = useState(0);
  const [watchlists, setWatchlists] = useState<Watchlist[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<Status>(null);
  const [activeWatchlist, setActiveWatchlist] = useState<number | null>(null);
  const [apartments, setApartments] = useState<Apartment[]>([]);

  const [form, setForm] = useState({
    location: "",
    minSize: 40,
    maxSize: 60,
    targetYield: 8,
  });

  const disabled = useMemo(() => chatId === 0, [chatId]);

  const notify = useCallback((message: string, tone: "success" | "error" = "success") => {
    setStatus({ message, tone });
    setTimeout(() => setStatus(null), 4000);
  }, []);

  const handleLoadWatchlists = useCallback(async () => {
    if (disabled) {
      notify("Please provide a chat id", "error");
      return;
    }
    setLoading(true);
    try {
      const response = await fetch(`${API_BASE}/api/watchlists?chat_id=${chatId}`);
      if (!response.ok) throw new Error("Failed to load watchlists");
      const payload = (await response.json()) as WatchlistsResponse;
      setWatchlists(payload.data.watchlists);
      notify(`Loaded ${payload.data.watchlists.length} watchlists`);
    } catch (error) {
      console.error(error);
      notify("Unable to load watchlists", "error");
    } finally {
      setLoading(false);
    }
  }, [chatId, disabled, notify]);

  const handleSubscribe = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (disabled) {
        notify("Please provide a chat id", "error");
        return;
      }
      setLoading(true);
      try {
        const response = await fetch(`${API_BASE}/api/watchlists`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            chat_id: chatId,
            location: form.location,
            min_size: form.minSize,
            max_size: form.maxSize,
            target_yield: form.targetYield,
          }),
        });
        if (!response.ok) throw new Error("Failed to subscribe");
        const payload = (await response.json()) as ApiResponse<Watchlist>;
        setWatchlists((previous) => [payload.data, ...previous]);
        notify(`Subscribed to ${payload.data.location_name}`);
      } catch (error) {
        console.error(error);
        notify("Unable to subscribe", "error");
      } finally {
        setLoading(false);
      }
    },
    [chatId, disabled, form.location, form.maxSize, form.minSize, form.targetYield, notify]
  );

  const handleDelete = useCallback(
    async (watchlistId: number) => {
      if (disabled) {
        notify("Please provide a chat id", "error");
        return;
      }
      try {
        const response = await fetch(
          `${API_BASE}/api/watchlists/${watchlistId}?chat_id=${chatId}`,
          { method: "DELETE" }
        );
        if (!response.ok) throw new Error("Unable to delete watchlist");
        setWatchlists((prev) => prev.filter((watchlist) => watchlist.id !== watchlistId));
        notify("Watchlist deleted");
      } catch (error) {
        console.error(error);
        notify("Unable to delete watchlist", "error");
      }
    },
    [chatId, disabled, notify]
  );

  const fetchApartments = useCallback(
    async (watchlistId: number, matching: boolean) => {
      if (disabled) {
        notify("Please provide a chat id", "error");
        return;
      }
      setLoading(true);
      try {
        const endpoint = matching ? "matching" : "apartments";
        const response = await fetch(
          `${API_BASE}/api/watchlists/${watchlistId}/${endpoint}?chat_id=${chatId}`
        );
        if (!response.ok) throw new Error("Failed to fetch apartments");
        const payload = (await response.json()) as ApartmentsResponse;
        setApartments(payload.data.apartments);
        setActiveWatchlist(watchlistId);
        notify(`Loaded ${payload.data.apartments.length} apartments`);
      } catch (error) {
        console.error(error);
        notify("Unable to fetch apartments", "error");
      } finally {
        setLoading(false);
      }
    },
    [chatId, disabled, notify]
  );

  return (
    <main>
      <h1>Apato Web Console</h1>
      <section>
        <h2>Identify User</h2>
        <label>
          Chat ID
          <input
            type="number"
            value={chatId === 0 ? "" : chatId}
            onChange={(event) => setChatId(Number(event.target.value))}
            placeholder="Enter your Telegram chat id"
          />
        </label>
        <button onClick={handleLoadWatchlists} disabled={disabled || loading}>
          Load Watchlists
        </button>
        {status && (
          <div className="status" style={{ color: status.tone === "error" ? "#b91c1c" : "#0f766e" }}>
            {status.message}
          </div>
        )}
      </section>

      <section>
        <h2>Create / Update Subscription</h2>
        <form onSubmit={handleSubscribe}>
          <label>
            Location Query
            <input
              value={form.location}
              onChange={(event) => setForm((prev) => ({ ...prev, location: event.target.value }))}
              placeholder="Zip code or area"
              required
            />
          </label>
          <label>
            Min Size (m²)
            <input
              type="number"
              value={form.minSize}
              onChange={(event) => setForm((prev) => ({ ...prev, minSize: Number(event.target.value) }))}
              required
            />
          </label>
          <label>
            Max Size (m²)
            <input
              type="number"
              value={form.maxSize}
              onChange={(event) => setForm((prev) => ({ ...prev, maxSize: Number(event.target.value) }))}
              required
            />
          </label>
          <label>
            Target Yield (%)
            <input
              type="number"
              value={form.targetYield}
              onChange={(event) => setForm((prev) => ({ ...prev, targetYield: Number(event.target.value) }))}
              required
            />
          </label>
          <button type="submit" disabled={disabled || loading}>
            Save Watchlist
          </button>
        </form>
      </section>

      <section>
        <h2>Your Watchlists</h2>
        {watchlists.length === 0 ? (
          <p>No watchlists loaded yet.</p>
        ) : (
          watchlists.map((watchlist) => (
            <div key={watchlist.id} className="watchlist-card">
              <strong>{watchlist.location_name}</strong>
              <div>
                Target yield: {formatNumber(watchlist.target_yield, 2)}%
              </div>
              <div>
                Size range: {watchlist.target_size_min ?? "-"} – {watchlist.target_size_max ?? "-"} m²
              </div>
              <div className="watchlist-actions">
                <button onClick={() => fetchApartments(watchlist.id, false)} disabled={loading}>
                  View All
                </button>
                <button onClick={() => fetchApartments(watchlist.id, true)} disabled={loading}>
                  View Matching
                </button>
                <button onClick={() => handleDelete(watchlist.id)} disabled={loading}>
                  Delete
                </button>
              </div>
            </div>
          ))
        )}
      </section>

      {activeWatchlist !== null && (
        <section>
          <h2>Apartments for Watchlist {activeWatchlist}</h2>
          {apartments.length === 0 ? (
            <p>No apartments found yet.</p>
          ) : (
            apartments.map((apartment) => (
              <div key={apartment.id} className="apartment-entry">
                <strong>{apartment.location_name ?? "Unknown location"}</strong>
                <div>Size: {formatNumber(apartment.size, 1)} m²</div>
                <div>Price: {formatCurrency(apartment.price)}</div>
                <div>Estimated rent: {formatCurrency(apartment.rent)}</div>
                <div>Yield: {formatNumber(apartment.estimated_yield, 2)}%</div>
                <div>Added: {formatDate(apartment.created_at)}</div>
                {apartment.url && (
                  <a href={apartment.url} target="_blank" rel="noreferrer">
                    Open listing
                  </a>
                )}
              </div>
            ))
          )}
        </section>
      )}
    </main>
  );
};

export default App;
